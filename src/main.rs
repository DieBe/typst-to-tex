mod diagnostics;
mod elems;
mod eval;
mod world;

use std::collections::HashMap;
use std::env::current_dir;
use std::io::Write;
use std::process::{Command, Stdio};

use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::anyhow;
use color_eyre::eyre::bail;
use elems::from_native;
use facet::Facet;
use itertools::Itertools;
use rand::random;
use regex::Regex;

use color_eyre::{eyre::Context as _, Result};
use typst::comemo::Track;
use typst::diag::SourceDiagnostic;
use typst::engine::Engine;
use typst::engine::Route;
use typst::engine::Sink;
use typst::engine::Traced;
use typst::foundations::Content;
use typst::foundations::NativeElement;
use typst::foundations::Smart;
use typst::foundations::StyleChain;
use typst::foundations::Styles;
use typst::introspection::Introspector;
use typst::layout::Abs;
use typst::layout::Em;
use typst::layout::GridChild;
use typst::layout::GridItem;
use typst::layout::Margin;
use typst::layout::PageElem;
use typst::layout::Ratio;
use typst::layout::Rel;
use typst::layout::Sides;
use typst::model::FigureElem;
use typst::model::Supplement;
use typst::syntax::Span;
use typst::text::RawContent;
use typst::World;
use typst::ROUTINES;
use typst_layout::layout_document;
use typst_pdf::pdf;
use typst_pdf::PdfOptions;

use crate::diagnostics::Diagnostics;
use crate::world::TypstWrapperWorld;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MinorIssueKind {
    SingleSmartQuote,
    IgnoredElem,
}

enum SmartQuoteState {
    Open,
    Closed,
}

struct Context<'a> {
    /// A collection of figures with the "wild" supplement which can be
    /// substituted in raw latex blocks, indexed by the label attached to
    /// them.
    wild_figures: &'a mut HashMap<String, Utf8PathBuf>,
    config: &'a Config,
    diags: &'a mut Diagnostics,
    engine: &'a mut Engine<'a>,
    generated_dir: &'a Utf8PathBuf,

    /// Tracks the state of " smart quotes to emit the appropriate latex quotes.
    /// ' must still be handled manually for now since they can also appear in
    /// posessives
    last_smart_quote: SmartQuoteState,

    /// These are issues that could in theory lead to a non-ideal result but which
    /// we don't want to warn for in the common case. Instead, we'll just report
    /// a summary of these at the end of the compilation process
    minor_issues: HashMap<MinorIssueKind, Vec<SourceDiagnostic>>,

    input: &'a Utf8PathBuf,

    eval_result: Option<HashMap<String, String>>,

    world: &'a TypstWrapperWorld,
    pandoc_preamble: Option<String>,
}

#[derive(Facet)]
pub struct Config {
    /// The main file containing the typst content you want to transpile to latex.
    /// If you are using a template with lots of styling, you probably don't want this
    /// to be your main file.
    ///
    /// For example, you probably want to make your main file be
    /// ```typst
    /// #show: acmart.with(
    ///     ...
    /// )
    /// #include "content.typ"
    /// ```
    /// where `content.typ` contains your actual document text.
    content_main: Utf8PathBuf,
    template: Utf8PathBuf,

    /// If you use the `ttt-eval` system for stateful queries, this is the file to use
    /// as the main file for evals. Generally you want this to be your project main file,
    /// not your content main
    eval_main: Option<Utf8PathBuf>,

    #[facet(default = "[t]")]
    figure_placement: String,

    /// The wrapper to use around inline typst content to make it look inlines.
    /// This seems to need tweaking on a per-template basis but the default here
    /// can serve as a starting point
    #[facet(default = "\\raisebox{-5pt}[1em]")]
    inline_wrapper: String,

    /// Optional file containing Typst preamble (imports, function definitions)
    /// that will be prepended when converting math through pandoc.
    pandoc_preamble: Option<Utf8PathBuf>,
}

/// Common arguments of compile, watch, and query.
#[derive(Debug, Clone, Parser, Default)]
pub struct CompileArgs {
    /// Report all minor issues with full diagnostics instead of just a summary at the
    /// end
    #[clap(long = "minor")]
    pub w_minor: bool,
}

fn eval_typst(world: &dyn World, diags: &mut Diagnostics) -> Result<Content> {
    let main_id = world.main();
    let main = world.source(main_id).context("Did not find a main")?;
    let traced = Traced::new(Span::from_range(main_id, 0..main.text().as_bytes().len()));
    let mut sink = Sink::new();

    let module = match typst_eval::eval(
        &ROUTINES,
        world.track(),
        traced.track(),
        sink.track_mut(),
        Route::default().track(),
        &main,
    ) {
        Ok(module) => module,
        Err(e) => {
            for diag in e {
                diags.push(diag);
            }

            bail!("Typst eval failed")
        }
    };

    let content = module.content();

    Ok(content)
}

fn label_to_supplement(l: &str) -> Option<String> {
    match l.split(":").next() {
        None => None,
        Some("fig") => Some("Fig.".to_string()),
        Some("sec") => Some("Section".to_string()),
        Some("lst") => Some("Listing".to_string()),
        Some("tab") => Some("Tab.".to_string()),
        _other => {
            // println!("Unknown label supplement, assuming citation: {other:?}");
            None
        }
    }
}

fn compile_subcontent(
    mut inner_content: Content,
    sc: StyleChain,
    engine: &mut Engine,
    generated_dir: &Utf8PathBuf,
    target_width_pt: Option<f64>,
) -> Result<Utf8PathBuf> {
    let filename = format!("{}.pdf", random::<u64>());
    let include_file = Utf8PathBuf::from("generated").join(&filename);
    let output_file = generated_dir.join(filename);

    let mut styles = Styles::new();
    match target_width_pt {
        Some(width_pt) => styles.set(
            PageElem::width,
            Smart::Custom(typst::layout::Length {
                abs: Abs::pt(width_pt),
                em: Em::zero(),
            }),
        ),
        None => styles.set(PageElem::width, Smart::Auto),
    }
    styles.set(PageElem::height, Smart::Auto);
    styles.set(
        PageElem::margin,
        Margin {
            sides: Sides {
                left: Some(Smart::Custom(Rel::zero())),
                right: Some(Smart::Custom(Rel::zero())),
                top: Some(Smart::Custom(Rel {
                    rel: Ratio::zero(),
                    abs: typst::layout::Length {
                        abs: Abs::pt(5.),
                        em: Em::zero(),
                    },
                })),
                bottom: Some(Smart::Custom(Rel {
                    rel: Ratio::zero(),
                    abs: typst::layout::Length {
                        abs: Abs::pt(5.),
                        em: Em::zero(),
                    },
                })),
            },
            two_sided: None,
        },
    );

    let sc = sc.chain(&styles);
    inner_content.materialize(sc);

    for _ in 1..5 {
        layout_document(engine, &mut inner_content, sc).unwrap();
    }
    let doc = layout_document(engine, &mut inner_content, sc).unwrap();

    let pdf = pdf(&doc, &PdfOptions::default()).unwrap();

    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create dir {parent}"))?;
    }

    std::fs::write(&output_file, pdf)
        .with_context(|| format!("Failed to write pdf to {output_file}"))?;

    Ok(include_file)
}

pub enum TexBlock {
    String(String),
    Label(String),
    Ref(String),
    Figure {
        content_file: Utf8PathBuf,
        caption: Option<Box<TexBlock>>,
        supplement: Option<String>,
    },
    Footnote(Box<TexBlock>),
    /// Math converted to a PDF file (fallback when pandoc fails)
    MathPdf(Utf8PathBuf),
    /// Math converted to LaTeX via pandoc
    MathLatex(std::string::String),
    InlineCode(String),
    CodeBlock {
        code: String,
        language: Option<String>,
    },
    Seq(Vec<TexBlock>),
    RawString(String),
    Maybe(Option<Box<TexBlock>>),
    Nothing,
}

impl TexBlock {
    fn emit(&self, wild_figures: &HashMap<String, Utf8PathBuf>, config: &Config) -> String {
        match self {
            TexBlock::String(s) => s.clone(),
            TexBlock::RawString(s) => {
                let mut s = s.clone();
                for (label, file) in wild_figures {
                    let repl = format!("#wild:{label}#");
                    s = s.replace(&repl, file.as_str());
                }
                s
            }
            TexBlock::Label(l) => {
                format!("\\label{{{l}}}")
            }
            TexBlock::Figure {
                content_file,
                caption,
                supplement,
            } => {
                let placement = &config.figure_placement;
                let environment = match supplement {
                    Some(s) if s == "Listing" => "Listing",
                    Some(s) if s == "Tab." => "table",
                    _ => "figure"
                };
                indoc::formatdoc!(
                    r#"
                    \begin{{{environment}}}{placement}
                        \centering
                        \maxsizebox{{\textwidth}}{{!}}{{\includegraphics{{{content_file}}}}}
                        {caption}
                    \end{{{environment}}}
                    "#,
                    caption = if let Some(caption) =
                        caption.as_ref().map(|s| s.emit(wild_figures, config))
                    {
                        format!(r"\caption{{{caption}}}")
                    } else {
                        String::new()
                    }
                )
            }
            TexBlock::MathPdf(pdf) => {
                let inline_wrapper = &config.inline_wrapper;
                format!(r#"{inline_wrapper}{{\includegraphics{{{pdf}}}}}"#)
            }
            TexBlock::MathLatex(latex) => latex.clone(),
            TexBlock::InlineCode(code) => {
                format!(r#"\texttt{{{}}}"#, escape_latex_inline_code(code))
            }
            TexBlock::CodeBlock { code, language } => {
                let lang_opt = language
                    .as_ref()
                    .map(|lang| format!("[language={lang}]"))
                    .unwrap_or_default();

                format!(
                    "\\begin{{lstlisting}}{lang_opt}\n{code}\n\\end{{lstlisting}}\n"
                )
            }
            TexBlock::Ref(l) => {
                let sup = match label_to_supplement(l) {
                    Some(sup) => sup,
                    None => {
                        return format!("~\\cite{{{l}}}");
                    }
                };

                format!("{sup} \\ref{{{l}}}")
            }
            TexBlock::Footnote(f) => {
                format!("\\footnote{{{}}}", f.emit(wild_figures, config))
            }
            TexBlock::Seq(inner) => inner.iter().map(|i| i.emit(wild_figures, config)).join(""),
            TexBlock::Maybe(inner) => inner
                .as_ref()
                .map(|inner| inner.emit(wild_figures, config))
                .unwrap_or_else(|| String::new()),
            TexBlock::Nothing => String::new(),
        }
    }
}

fn escape_latex_inline_code(input: &str) -> String {
    input
        .replace('\\', r#"\textbackslash{}"#)
        .replace('{', r#"\{"#)
        .replace('}', r#"\}"#)
        .replace('%', r#"\%"#)
        .replace('_', r#"\_"#)
        .replace('&', r#"\&"#)
        .replace('#', r#"\#"#)
        .replace('$', r#"\$"#)
        .replace('^', r#"\^{}"#)
        .replace('~', r#"\~{}"#)
}

fn map_listings_language(lang: &str) -> Option<String> {
    let mapped = match lang.trim().to_ascii_lowercase().as_str() {
        "py" | "python" => Some("Python"),
        "rs" | "rust" => Some("Rust"),
        "c" => Some("C"),
        "cpp" | "c++" | "cc" | "cxx" => Some("C++"),
        "java" => Some("Java"),
        "go" => Some("Go"),
        "sql" => Some("SQL"),
        "bash" | "sh" | "shell" | "zsh" => Some("bash"),
        _ => None,
    };

    mapped.map(|s| s.to_string())
}

fn ensure_listings_preamble(mut source: String) -> String {
    if !source.contains("\\begin{lstlisting}") {
        return source;
    }

    if source.contains(r#"\usepackage{listings}"#) {
        return source;
    }

    let insertion = "\\usepackage{listings}\n\\lstset{basicstyle=\\ttfamily\\small,columns=fullflexible,keepspaces=true,showstringspaces=false}\n";

    if let Some(idx) = source.find(r#"\begin{document}"#) {
        source.insert_str(idx, insertion);
    } else {
        source.push('\n');
        source.push_str(insertion);
    }

    source
}

fn strip_balanced_macro_call(mut source: String, macro_name: &str) -> String {
    let marker = format!("#{macro_name}(");

    while let Some(start) = source.find(&marker) {
        let mut depth = 0usize;
        let mut end = None;

        let mut in_string = false;
        let mut escaped = false;

        for (idx, ch) in source[start + marker.len() - 1..].char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        continue;
                    }
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + marker.len() - 1 + idx + ch.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }

        let Some(mut end) = end else {
            break;
        };

        while end < source.len() {
            let tail = &source[end..];
            if let Some(rest) = tail.strip_prefix("\r\n") {
                end += 2;
                if rest.starts_with('\n') {
                    end += 1;
                }
                break;
            }
            if tail.starts_with('\n') {
                end += 1;
                break;
            }
            if tail.starts_with(' ') || tail.starts_with('\t') {
                end += 1;
                continue;
            }
            break;
        }

        source.replace_range(start..end, "");
    }

    source
}

fn preprocess_typst_source(source: &str) -> String {
    let local_import_re = Regex::new(r#"(?m)^\s*#import\s*"@local/([^":]+):[^"]+"\s*:[^\n]*\n?"#)
        .expect("local import regex should compile");
    let local_import_names: Vec<String> = local_import_re
        .captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|name| name.as_str().to_string()))
        .collect();

    let mut processed = local_import_re.replace_all(source, "").to_string();

    for name in local_import_names {
        let escaped = regex::escape(&name);
        let show_rule_re = Regex::new(&format!(r"(?m)^\s*#show\s*:\s*{escaped}\s*\n?"))
            .expect("show rule regex should compile");
        processed = show_rule_re.replace_all(&processed, "").to_string();
    }

    // Known template helper from hpclab package; this is layout-only metadata.
    processed = strip_balanced_macro_call(processed, "serieheader");

    processed
}

fn get_source_text(span: Span, world: &TypstWrapperWorld) -> Option<String> {
    let file_id = span.id()?;
    let source = world.source(file_id).ok()?;
    let range = source.range(span)?;
    Some(source.text()[range].to_string())
}

fn run_pandoc(typst_input: &str) -> Result<String> {
    let mut child = Command::new("pandoc")
        .args(["-f", "typst", "-t", "latex"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run pandoc. Is it installed?")?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(typst_input.as_bytes())
        .context("Failed to write to pandoc stdin")?;

    let output = child
        .wait_with_output()
        .context("Failed to wait for pandoc")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Pandoc failed: {stderr}");
    }

    Ok(String::from_utf8(output.stdout)
        .context("Pandoc output was not valid UTF-8")?
        .trim()
        .to_string())
}

fn try_pandoc_math(
    span: Span,
    preamble: Option<&str>,
    world: &TypstWrapperWorld,
) -> Result<String> {
    let source_text = get_source_text(span, world)
        .ok_or_else(|| anyhow!("Could not extract source text for equation"))?;

    let typst_input = match preamble {
        Some(preamble) => format!("{preamble}\n\n{source_text}"),
        None => source_text,
    };

    run_pandoc(&typst_input)
}

fn into_latex(mut content: Content, sc: StyleChain, ctx: &mut Context) -> Result<TexBlock> {
    content.materialize(sc);

    let label_text = content.label().map(|l| l.resolve().to_string());
    let label = TexBlock::Maybe(label_text.clone().map(TexBlock::Label).map(Box::new));
    let native = from_native(content.clone());

    macro_rules! warn_here {
        ($message:expr) => {
            ctx.diags
                .push(SourceDiagnostic::warning(content.span(), format!($message)))
        };
    }

    let result = match native {
        elems::Elem::HideElem(_) => TexBlock::Nothing,
        elems::Elem::CiteElem(cite) => {
            TexBlock::RawString(format!("\\cite{{{}}}", cite.key.resolve()))
        }
        elems::Elem::CiteGroup(group) => {
            let keys: Vec<String> = group
                .children
                .iter()
                .map(|c| c.key.resolve().to_string())
                .collect();
            TexBlock::RawString(format!("\\cite{{{}}}", keys.join(",")))
        }
        elems::Elem::EmphElem(emph_elem) => TexBlock::String(format!(
            "\\textit{{{}}}",
            into_latex(emph_elem.body, sc, ctx,)?.emit(ctx.wild_figures, ctx.config)
        )),
        elems::Elem::StrongElem(strong_elem) => TexBlock::String(format!(
            "\\textbf{{{}}}",
            into_latex(strong_elem.body, sc, ctx,)?.emit(ctx.wild_figures, ctx.config)
        )),
        elems::Elem::EnumElem(enum_elem) => {
            let items = enum_elem
                .children
                .iter()
                .map(|item| {
                    let body = into_latex(item.body.clone(), sc, ctx)?;
                    Ok(format!("\\item {}", body.emit(ctx.wild_figures, ctx.config)))
                })
                .collect::<Result<Vec<_>>>()?;
            TexBlock::RawString(format!(
                "\\begin{{enumerate}}\n{}\n\\end{{enumerate}}",
                items.join("\n")
            ))
        }
        elems::Elem::FigureElem(figure_elem) => {
            let content = FigureElem::new(figure_elem.body.clone());

            // Keep a concrete width so `image(..., width: 100%)` inside figures can resolve.
            let filename = compile_subcontent(
                content.pack(),
                sc,
                ctx.engine,
                ctx.generated_dir,
                Some(469.47),
            )?;

            if let Some(Some(Supplement::Content(c))) =
                figure_elem.supplement.get_ref(sc).clone().custom()
            {
                if c.plain_text() == "wild" {
                    if let Some(label) = label_text {
                        ctx.wild_figures.insert(label, filename);
                        return Ok(TexBlock::Nothing);
                    } else {
                        panic!("Found a wild supplement figure that did not have a label attached")
                    }
                }
            }
            let caption = figure_elem
                .caption
                .get_ref(sc)
                .as_ref()
                .map(|cap| into_latex(cap.body.clone(), sc, ctx))
                .transpose()?
                .map(|cap| TexBlock::Seq(vec![cap, label]))
                .map(Box::new);

            TexBlock::Figure {
                content_file: filename,
                caption,
                supplement: label_text.and_then(|s| label_to_supplement(&s)),
            }
        }
        elems::Elem::EquationElem(eq) => {
            match try_pandoc_math(content.span(), ctx.pandoc_preamble.as_deref(), ctx.world) {
                Ok(latex) => TexBlock::MathLatex(latex),
                Err(e) => {
                    warn_here!("Pandoc math conversion failed ({e}), falling back to PDF");
                    let filename =
                        compile_subcontent(eq.pack(), sc, ctx.engine, ctx.generated_dir, None)?;
                    TexBlock::MathPdf(filename)
                }
            }
        }
        elems::Elem::FootnoteElem(f) => {
            if let Some(body) = f.body_content() {
                TexBlock::Footnote(Box::new(into_latex(body.clone(), sc, ctx)?))
            } else {
                warn_here!("Found an empty footnote, ignoring");
                TexBlock::Nothing
            }
        }
        elems::Elem::HeadingElem(heading_elem) => {
            let level = heading_elem.offset.get_ref(sc);
            let depth = heading_elem.depth.get_ref(sc);

            let h = match depth.get() + level {
                1 => "section",
                2 => "subsection",
                3 => "subsubsection",
                _ => {
                    warn_here!(
                        "More than 3 levels of sections is unsupported. Falling back on subsection"
                    );
                    "subsubsection"
                }
            };

            let heading = TexBlock::String(format!(
                "\\{h}{{{}}}",
                into_latex(heading_elem.body, sc, ctx)?.emit(ctx.wild_figures, ctx.config)
            ));

            TexBlock::Seq(vec![heading, label])
        }
        elems::Elem::LinkElem(link_elem) => match link_elem.dest {
            typst::model::LinkTarget::Dest(typst::model::Destination::Url(url)) => {
                TexBlock::String(format!("\\url{{{}}}", url.as_str()))
            }
            typst::model::LinkTarget::Dest(_) => {
                warn_here!("Ignoring link with non-url destination");
                TexBlock::Nothing
            }
            typst::model::LinkTarget::Label(_label) => {
                warn_here!("Ignoring link target label");
                TexBlock::Nothing
            }
        },
        elems::Elem::ListElem(list_elem) => {
            let items = list_elem
                .children
                .iter()
                .map(|item| {
                    let body = into_latex(item.body.clone(), sc, ctx)?;
                    Ok(format!("\\item {}", body.emit(ctx.wild_figures, ctx.config)))
                })
                .collect::<Result<Vec<_>>>()?;
            TexBlock::RawString(format!(
                "\\begin{{itemize}}\n{}\n\\end{{itemize}}",
                items.join("\n")
            ))
        }
        elems::Elem::GridElem(grid_elem) => {
            let mut converted_children = Vec::new();

            for child in &grid_elem.children {
                match child {
                    GridChild::Item(item) => {
                        if let GridItem::Cell(cell) = item {
                            converted_children.push(into_latex(cell.body.clone(), sc, ctx)?);
                        }
                    }
                    GridChild::Header(header) => {
                        for item in &header.children {
                            if let GridItem::Cell(cell) = item {
                                converted_children.push(into_latex(cell.body.clone(), sc, ctx)?);
                            }
                        }
                    }
                    GridChild::Footer(footer) => {
                        for item in &footer.children {
                            if let GridItem::Cell(cell) = item {
                                converted_children.push(into_latex(cell.body.clone(), sc, ctx)?);
                            }
                        }
                    }
                }
            }

            TexBlock::Seq(converted_children)
        }
        elems::Elem::ParElem(par_elem) => into_latex(par_elem.body, sc, ctx)?,
        elems::Elem::ParLineMarker(_) => TexBlock::Nothing,
        elems::Elem::ParbreakElem(_) => TexBlock::String("\n\n".to_string()),
        elems::Elem::QuoteElem(_quote_elem) => {
            warn_here!("Unsupported QuoteElem");
            TexBlock::Nothing
        }
        elems::Elem::RefElem(ref_elem) => TexBlock::Ref(ref_elem.target.resolve().to_string()),
        elems::Elem::HighlightElem(_highlight_elem) => {
            warn_here!("Unsupported HighlightElem");
            TexBlock::Nothing
        }
        elems::Elem::LinebreakElem(_linebreak_elem) => {
            warn_here!("Unsupported LinebreakElem");
            TexBlock::Nothing
        }
        elems::Elem::TableElem(_table_elem) => {
            warn_here!("Unsupported TableElem");
            TexBlock::Nothing
        }
        elems::Elem::RawElem(raw_elem) => {
            let lang = raw_elem.lang.get_ref(sc).as_ref().map(|s| s.as_str());
            let is_block = *raw_elem.block.get_ref(sc);
            let raw_content = match &raw_elem.text {
                RawContent::Text(s) => s.to_string(),
                RawContent::Lines(eco_vec) => eco_vec.iter().map(|(s, _)| s.to_string()).join("\n"),
            };
            if lang == Some("latexraw") {
                TexBlock::RawString(raw_content)
            } else if lang == Some("ttt-eval") {
                if let Some(eval_result) = &ctx.eval_result {
                    TexBlock::String(eval_result.get(&raw_content).cloned().unwrap_or_else(|| {
                        warn_here!("Did not find an entry for {raw_content} in the eval result");
                        raw_content.clone()
                    }))
                } else {
                    warn_here!("Found a ttt-eval block but no eval result is available. Did you set eval_main in ttt.toml?");
                    TexBlock::String(raw_content.clone())
                }
            } else {
                if is_block {
                    TexBlock::CodeBlock {
                        code: raw_content,
                        language: lang.and_then(map_listings_language),
                    }
                } else {
                    TexBlock::InlineCode(raw_content)
                }
            }
        }
        elems::Elem::SmallcapsElem(_smallcaps_elem) => {
            warn_here!("Unsupported SmallcapsElem");
            TexBlock::Nothing
        }
        elems::Elem::SmartQuoteElem(elem) => {
            if *elem.double.get_ref(sc) {
                let (result, state) = match ctx.last_smart_quote {
                    SmartQuoteState::Open => ("''", SmartQuoteState::Closed),
                    SmartQuoteState::Closed => ("``", SmartQuoteState::Open),
                };
                ctx.last_smart_quote = state;
                TexBlock::RawString(result.to_string())
            } else {
                let diag = SourceDiagnostic::warning(
                    content.span(),
                    "Unhandled smart quote `'`, emitting '.",
                )
                .with_hint("You may want to edit this manually afterwards");

                ctx.minor_issues
                    .entry(MinorIssueKind::SingleSmartQuote)
                    .or_insert(vec![])
                    .push(diag);
                TexBlock::RawString("\'".to_string())
            }
        }
        elems::Elem::SpaceElem(_) => TexBlock::String(" ".to_string()),
        elems::Elem::StrikeElem(_strike_elem) => {
            warn_here!("Unsupported StrikeElem");
            TexBlock::Nothing
        }
        elems::Elem::SubElem(_sub_elem) => {
            warn_here!("Unsupported SubElem");
            TexBlock::Nothing
        }
        elems::Elem::SuperElem(_super_elem) => {
            warn_here!("Unsupported SuperElem");
            TexBlock::Nothing
        }
        elems::Elem::TextElem(text) => TexBlock::String(
            text.text.to_string()
                .replace("%", "\\%")
                .replace("_", "\\_")
                .replace("^", "\\^{}")
        ),
        elems::Elem::UnderlineElem(_underline_elem) => {
            warn_here!("Unsupported UnderlineElem");
            TexBlock::Nothing
        }
        elems::Elem::ContextElem(_context_elem) => {
            ctx.diags.push(SourceDiagnostic::warning(
                content.span(),
                "Context is not supported. Consider using #emit-latex",
            ));
            TexBlock::Nothing
        }
        elems::Elem::SequenceElem(s) => TexBlock::Seq(
            s.children
                .iter()
                .map(|content| into_latex(content.clone(), sc, ctx))
                .collect::<Result<_>>()?,
        ),
        elems::Elem::StyledElem(styled_elem) => {
            let sc = sc.chain(styled_elem.styles.as_slice());
            into_latex(styled_elem.child, sc, ctx)?
        }
        elems::Elem::SymbolElem(symbol_elem) => {
            TexBlock::String(symbol_elem.text.to_string())
        },
        elems::Elem::BoxElem(_) => {
            ctx.diags.push(SourceDiagnostic::warning(
                content.span(),
                "Unsupported box element",
            ));
            TexBlock::Nothing
        }
        elems::Elem::PagebreakElem(_) => {
            TexBlock::RawString("\\pagebreak".to_string())
        }
        elems::Elem::EnumItem(item) => {
            let body = into_latex(item.body.clone(), sc, ctx)?;
            TexBlock::RawString(format!(
                "\\begin{{enumerate}}\n\\item {}\n\\end{{enumerate}}",
                body.emit(ctx.wild_figures, ctx.config)
            ))
        }
        elems::Elem::ListItem(item) => {
            let body = into_latex(item.body.clone(), sc, ctx)?;
            TexBlock::RawString(format!(
                "\\begin{{itemize}}\n\\item {}\n\\end{{itemize}}",
                body.emit(ctx.wild_figures, ctx.config)
            ))
        }
        elems::Elem::Ignored => {
            let diag =
                SourceDiagnostic::warning(content.span(), "Encountered a fully ignored element.");

            ctx.minor_issues
                .entry(MinorIssueKind::IgnoredElem)
                .or_insert(vec![])
                .push(diag);

            TexBlock::Nothing
        }
    };
    Ok(result)
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = CompileArgs::parse();

    let config_content =
        std::fs::read_to_string("ttt.toml").context(format!("Failed to read ttt.toml"))?;

    let config = match facet_toml::from_str::<Config>(&config_content) {
        Ok(config) => config,
        // Not entirely sure why this is needed, but lifetimes get in the way without it
        Err(e) => return Err(anyhow!("{e:#}").into()),
    };

    let eval_result = config
        .eval_main
        .as_ref()
        .map(|file| eval::run_eval(&file))
        .transpose()?;

    let pandoc_preamble = config
        .pandoc_preamble
        .as_ref()
        .map(|path| std::fs::read_to_string(path))
        .transpose()
        .with_context(|| "Failed to read pandoc preamble file")?;

    let template = std::fs::read_to_string(&config.template)
        .with_context(|| format!("Failed to read template from {}", config.template))?;

    let main_source_raw = std::fs::read_to_string(&config.content_main)
        .with_context(|| format!("Failed to read {}", config.content_main))?;
    let main_source = preprocess_typst_source(&main_source_raw);

    let world = TypstWrapperWorld::new(
        current_dir()
            .with_context(|| "Failed to get current dir")?
            .to_str()
            .ok_or_else(|| anyhow!("Current dir was not a unicode path"))?
            .to_string(),
        &config.content_main,
        main_source,
    );

    let empty_introspector = Introspector::default();
    let traced = Traced::new(Span::detached());
    let mut sink = Sink::new();
    let mut engine = Engine {
        routines: &ROUTINES,
        world: (&world as &dyn World).track(),
        introspector: empty_introspector.track(),
        traced: traced.track(),
        sink: sink.track_mut(),
        route: Route::default(),
    };
    let mut diagnostics = Diagnostics::new();
    let mut wild_figures = HashMap::new();
    let generated_dir = config
        .content_main
        .parent()
        .map(|parent| parent.join("generated"))
        .unwrap_or_else(|| Utf8PathBuf::from("generated"));
    let ctx = &mut Context {
        wild_figures: &mut wild_figures,
        config: &config,
        diags: &mut diagnostics,
        engine: &mut engine,
        generated_dir: &generated_dir,
        last_smart_quote: SmartQuoteState::Closed,
        minor_issues: HashMap::new(),
        input: &config.content_main,
        eval_result,
        world: &world,
        pandoc_preamble,
    };

    let result = (|| {
        let content = eval_typst(&world, ctx.diags)?;

        let library = world.library();

        let sc = StyleChain::new(&library.styles);

        let latex = into_latex(content, sc, ctx)?;

        let latex_source = template.replace("%CONTENT%", &latex.emit(&ctx.wild_figures, &config));

        // Citation groups are easier to fix with regex than to look for them in the source
        let cite_fix_regex = Regex::new(r"(\s+~\\cite\{([^}]*)\})+").unwrap();
        let cite_body_regex = Regex::new(r"\s+~\\cite\{([^}]*)\}").unwrap();
        let latex_source =
            cite_fix_regex.replace_all(&latex_source, |captures: &regex::Captures<'_>| {
                let inner = captures.get(0).unwrap().as_str();
                let inner_caps = cite_body_regex.captures_iter(inner);
                format!(
                    r"~\cite{{{}}}",
                    inner_caps.map(|cap| cap.get(1).unwrap().as_str()).join(",")
                )
            });

        // Merge consecutive enumerate/itemize environments that come from individual EnumItem/ListItem elements
        let enum_merge_regex = Regex::new(r"\\end\{enumerate\}\s*\\begin\{enumerate\}").unwrap();
        let latex_source = enum_merge_regex.replace_all(&latex_source, "\n");
        let list_merge_regex = Regex::new(r"\\end\{itemize\}\s*\\begin\{itemize\}").unwrap();
        let latex_source = list_merge_regex.replace_all(&latex_source, "\n");

        let latex_source = ensure_listings_preamble(latex_source.to_string());

        let filename = format!("{}.tex", config.content_main);
        std::fs::write(filename, latex_source)
            .with_context(|| "Failed to write out/out.tex")?;

        Ok(())
    })();

    for (_issue, diags) in &ctx.minor_issues {
        if args.w_minor {
            for diag in diags {
                ctx.diags.push(diag.clone());
            }
            ctx.diags.report(&world)?;
        }
    }

    ctx.diags.report(&world)?;

    if !args.w_minor {
        if !ctx.minor_issues.is_empty() {
            eprintln!("There were a few minor issues:");
            eprintln!("These are probably fine, but we report them just in case. Rerun with `--minor` to see their full diagnostics");
        }
        for (issue, diags) in &ctx.minor_issues {
            match issue {
                MinorIssueKind::SingleSmartQuote => {
                    eprintln!("    Smart single quote: {}", diags.len())
                }
                MinorIssueKind::IgnoredElem => {
                    eprintln!("    Ignored element:    {}", diags.len())
                }
            }
        }
    }

    result
}
