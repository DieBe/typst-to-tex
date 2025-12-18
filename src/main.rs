mod elems;
mod world;

use std::collections::HashMap;
use std::env::current_dir;

use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::anyhow;
use elems::from_native;
use facet::Facet;
use itertools::Itertools;
use rand::random;
use regex::Regex;

use color_eyre::{eyre::Context, Result};
use typst::comemo::Track;
use typst::engine::Engine;
use typst::engine::Route;
use typst::engine::Sink;
use typst::engine::Traced;
use typst::foundations::Content;
use typst::foundations::NativeElement;
use typst::foundations::Smart;
use typst::foundations::StyleChain;
use typst::introspection::Introspector;
use typst::layout::Abs;
use typst::layout::Em;
use typst::layout::Margin;
use typst::layout::PageElem;
use typst::layout::Ratio;
use typst::layout::Rel;
use typst::layout::Sides;
use typst::model::FigureElem;
use typst::model::Supplement;
use typst::syntax::Span;
use typst::text::RawContent;
use typst::Library;
use typst::World;
use typst::ROUTINES;
use typst_layout::layout_document;
use typst_pdf::pdf;
use typst_pdf::PdfOptions;

use crate::world::TypstWrapperWorld;

#[derive(Facet)]
pub struct Config {
    template: Utf8PathBuf,

    #[facet(default = "[t]")]
    figure_placement: String,

    /// The wrapper to use around inline typst content to make it look inlines.
    /// This seems to need tweaking on a per-template basis but the default here
    /// can serve as a starting point
    #[facet(default = "\\raisebox{-0.5em}[1em]")]
    inline_wrapper: String,
}

/// Common arguments of compile, watch, and query.
#[derive(Debug, Clone, Parser, Default)]
pub struct CompileArgs {
    pub input: Utf8PathBuf,
}

fn eval_typst(world: &dyn World) -> Result<Content> {
    let main = world.main();
    let main = world.source(main).context("Did not find a main")?;
    let traced = Traced::new(Span::detached());
    let mut sink = Sink::new();

    let module = typst_eval::eval(
        &ROUTINES,
        world.track(),
        traced.track(),
        sink.track_mut(),
        Route::default().track(),
        &main,
    )
    .unwrap();

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
) -> Result<Utf8PathBuf> {
    let filename = format!("{}.pdf", random::<u64>());
    let output_file = Utf8PathBuf::from("generated").join(filename);

    println!("Compiling {inner_content:?}");

    let styles = &[
        PageElem::set_width(Smart::Auto).wrap(),
        PageElem::set_height(Smart::Auto).wrap(),
        PageElem::set_margin(Margin::splat(Some(Smart::Custom(Rel::zero())))).wrap(),
        PageElem::set_margin(Margin {
            sides: Sides {
                left: Some(Smart::Custom(Rel::zero())),
                right: Some(Smart::Custom(Rel::zero())),
                top: Some(Smart::Custom(Rel {
                    rel: Ratio::zero(),
                    abs: typst::layout::Length {
                        abs: Abs::zero(),
                        em: Em::new(0.5),
                    },
                })),
                bottom: Some(Smart::Custom(Rel {
                    rel: Ratio::zero(),
                    abs: typst::layout::Length {
                        abs: Abs::zero(),
                        em: Em::new(0.5),
                    },
                })),
            },
            two_sided: None,
        })
        .wrap(),
    ];
    let sc = sc.chain(styles);
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

    Ok(output_file)
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
    Math(Utf8PathBuf),
    InlineCode(Utf8PathBuf),
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
                let supplement_override = match supplement {
                    Some(s) => format!(r"\renewcommand\figurename{{{}}}", s.to_string()),
                    None => "".to_string(),
                };
                indoc::formatdoc!(
                    r#"
                    \begin{{figure}}{placement}
                        {supplement_override}
                        \centering
                        \maxsizebox{{\textwidth}}{{!}}{{\includegraphics{{{content_file}}}}}
                        {caption}
                    \end{{figure}}
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
            TexBlock::Math(pdf) => {
                let inline_wrapper = &config.inline_wrapper;
                format!(r#"{inline_wrapper}{{\includegraphics{{{pdf}}}}}"#)
            }
            TexBlock::InlineCode(pdf) => {
                let inline_wrapper = &config.inline_wrapper;
                format!(r#"{inline_wrapper}{{\includegraphics{{{pdf}}}}}"#)
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

fn into_latex(
    mut content: Content,
    // A collection of figures with the "wild" supplement which can be
    // substituted in raw latex blocks, indexed by the // label attached to
    // them.
    wild_figures: &mut HashMap<String, Utf8PathBuf>,
    config: &Config,
    sc: StyleChain,
    world: &dyn World,
    engine: &mut Engine,
) -> Result<TexBlock> {
    content.materialize(sc);

    let label_text = content.label().map(|l| l.resolve().to_string());
    let label = TexBlock::Maybe(label_text.clone().map(TexBlock::Label).map(Box::new));
    let native = from_native(content.clone());

    let result = match native {
        elems::Elem::HideElem(_) => TexBlock::Nothing,
        elems::Elem::CiteElem(_) | elems::Elem::CiteGroup(_) => {
            println!("Unsupported CiteElem");
            TexBlock::Nothing
        }
        elems::Elem::EmphElem(emph_elem) => TexBlock::String(format!(
            "\\textit{{{}}}",
            into_latex(emph_elem.body, wild_figures, config, sc, world, engine)?
                .emit(wild_figures, config)
        )),
        elems::Elem::StrongElem(strong_elem) => TexBlock::String(format!(
            "\\textbf{{{}}}",
            into_latex(strong_elem.body, wild_figures, config, sc, world, engine)?
                .emit(wild_figures, config)
        )),
        elems::Elem::EnumElem(_enum_elem) => {
            println!("Unsupported EnumElem");
            TexBlock::Nothing
        }
        elems::Elem::FigureElem(figure_elem) => {
            let content = FigureElem::new(figure_elem.body.clone());

            let filename = compile_subcontent(content.pack(), sc, engine)?;

            if let Some(Some(Supplement::Content(c))) = figure_elem.supplement(sc).clone().custom()
            {
                if c.plain_text() == "wild" {
                    if let Some(label) = label_text {
                        wild_figures.insert(label, filename);
                        return Ok(TexBlock::Nothing);
                    } else {
                        panic!("Found a wild supplement figure that did not have a label attached")
                    }
                }
            }
            let caption = figure_elem
                .caption(sc)
                .as_ref()
                .map(|cap| into_latex(cap.body.clone(), wild_figures, config, sc, world, engine))
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
            let filename = compile_subcontent(eq.pack(), sc, engine)?;

            TexBlock::Math(filename)
        }
        elems::Elem::FootnoteElem(f) => {
            if let Some(body) = f.body_content() {
                TexBlock::Footnote(Box::new(into_latex(
                    body.clone(),
                    wild_figures,
                    config,
                    sc,
                    world,
                    engine,
                )?))
            } else {
                println!("Empty footnote???");
                TexBlock::Nothing
            }
        }
        elems::Elem::HeadingElem(heading_elem) => {
            let level = heading_elem.offset(sc);
            let depth = heading_elem.depth(sc);

            let h = match depth.get() + level {
                1 => "section",
                2 => "subsection",
                3 => "subsubsection",
                _ => {
                    println!("More than 3 levels of sections is unsupported. Using subsection");
                    "subsubsection"
                }
            };

            let heading = TexBlock::String(format!(
                "\\{h}{{{}}}",
                into_latex(heading_elem.body, wild_figures, config, sc, world, engine)?
                    .emit(wild_figures, config)
            ));

            TexBlock::Seq(vec![heading, label])
        }
        elems::Elem::LinkElem(link_elem) => match link_elem.dest {
            typst::model::LinkTarget::Dest(typst::model::Destination::Url(url)) => {
                TexBlock::String(format!("\\url{{{}}}", url.as_str()))
            }
            typst::model::LinkTarget::Dest(_) => {
                println!("Ignoring link with non-url destination");
                TexBlock::Nothing
            }
            typst::model::LinkTarget::Label(_label) => {
                println!("Ignoring link target label");
                TexBlock::Nothing
            }
        },
        elems::Elem::ListElem(_list_elem) => {
            println!("Unsupported ListElem");
            TexBlock::Nothing
        }
        elems::Elem::ParElem(par_elem) => {
            into_latex(par_elem.body, wild_figures, config, sc, world, engine)?
        }
        elems::Elem::ParLineMarker(_) => TexBlock::Nothing,
        elems::Elem::ParbreakElem(_) => TexBlock::String("\n\n".to_string()),
        elems::Elem::QuoteElem(_quote_elem) => {
            println!("Unsupported QuoteElem");
            TexBlock::Nothing
        }
        elems::Elem::RefElem(ref_elem) => TexBlock::Ref(ref_elem.target.resolve().to_string()),
        elems::Elem::HighlightElem(_highlight_elem) => {
            println!("Unsupported HighlightElem");
            TexBlock::Nothing
        }
        elems::Elem::LinebreakElem(_linebreak_elem) => {
            println!("Unsupported LinebreakElem");
            TexBlock::Nothing
        }
        elems::Elem::TableElem(_table_elem) => {
            println!("Unsupported TableElem");
            TexBlock::Nothing
        }
        elems::Elem::RawElem(raw_elem) => {
            if raw_elem.lang(sc).as_ref().map(|s| s.as_str()) == Some("latexraw") {
                let raw_content = match raw_elem.text {
                    RawContent::Text(s) => s.to_string(),
                    RawContent::Lines(eco_vec) => {
                        eco_vec.iter().map(|(s, _)| s.to_string()).join("\n")
                    }
                };
                TexBlock::RawString(raw_content)
            } else {
                let filename = compile_subcontent(raw_elem.pack(), sc, engine)?;

                TexBlock::InlineCode(filename)
            }
        }
        elems::Elem::SmallcapsElem(_smallcaps_elem) => {
            println!("Unsupported SmallcapsElem");
            TexBlock::Nothing
        }
        elems::Elem::SmartQuoteElem(elem) => {
            println!("Note: SmartQuoteElem is replaced with ('\"'), you may have to edit this manually for best looks");
            if elem.double(sc) {
                TexBlock::RawString("\"".to_string())
            } else {
                TexBlock::RawString("\'".to_string())
            }
        }
        elems::Elem::SpaceElem(_) => TexBlock::String(" ".to_string()),
        elems::Elem::StrikeElem(_strike_elem) => {
            println!("Unsupported StrikeElem");
            TexBlock::Nothing
        }
        elems::Elem::SubElem(_sub_elem) => {
            println!("Unsupported SubElem");
            TexBlock::Nothing
        }
        elems::Elem::SuperElem(_super_elem) => {
            println!("Unsupported SuperElem");
            TexBlock::Nothing
        }
        elems::Elem::TextElem(text) => TexBlock::String(text.text.to_string()),
        elems::Elem::UnderlineElem(_underline_elem) => {
            println!("Unsupported UnderlineElem");
            TexBlock::Nothing
        }
        elems::Elem::ContextElem(_context_elem) => {
            println!("Context is not supported. Consider using #emit-latex");
            TexBlock::Nothing
        }
        elems::Elem::SequenceElem(s) => TexBlock::Seq(
            s.children
                .iter()
                .map(|content| into_latex(content.clone(), wild_figures, config, sc, world, engine))
                .collect::<Result<_>>()?,
        ),
        elems::Elem::StyledElem(styled_elem) => {
            let sc = sc.chain(styled_elem.styles.as_slice());
            into_latex(styled_elem.child, wild_figures, config, sc, world, engine)?
        }
        elems::Elem::SymbolElem(_symbol_elem) => TexBlock::Nothing,
        elems::Elem::BoxElem(box_elem) => {
            println!("unsupported box elem {box_elem:?}");
            TexBlock::Nothing
        }
        elems::Elem::Ignored => TexBlock::Nothing,
    };
    Ok(result)
}

fn main() -> Result<()> {
    // Parse command line arguments
    let mut args = CompileArgs::parse();

    let config_content =
        std::fs::read_to_string("ttt.toml").context(format!("Failed to read ttt.toml"))?;

    let config = match facet_toml::from_str::<Config>(&config_content) {
        Ok(config) => config,
        // Not entirely sure why this is needed, but lifetimes get in the way without it
        Err(e) => return Err(anyhow!("{e:#}").into()),
    };

    let template = std::fs::read_to_string(&config.template)
        .with_context(|| format!("Failed to read template from {}", config.template))?;

    let world = TypstWrapperWorld::new(
        current_dir()
            .with_context(|| "Failed to get current dir")?
            .to_str()
            .ok_or_else(|| anyhow!("Current dir was not a unicode path"))?
            .to_string(),
        args.input.clone().into_string(),
    );

    let content = eval_typst(&world)?;

    let library = Library::builder().build();
    let sc = StyleChain::new(&library.styles);

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

    let mut wild_figures = HashMap::new();
    let latex = into_latex(content, &mut wild_figures, &config, sc, &world, &mut engine)?;

    let latex_source = template.replace("%CONTENT%", &latex.emit(&wild_figures, &config));

    // Citation groups are easier to fix with regex than to look for them in the source
    let cite_fix_regex = Regex::new(r"(\s+~\\cite\{([^}]*)\})+").unwrap();
    let cite_body_regex = Regex::new(r"\s+~\\cite\{([^}]*)\}").unwrap();
    let latex_source =
        cite_fix_regex.replace_all(&latex_source, |captures: &regex::Captures<'_>| {
            let inner = captures.get(0).unwrap().as_str();
            let inner_caps = cite_body_regex.captures_iter(inner);
            format!(
                r"\cite{{{}}}",
                inner_caps.map(|cap| cap.get(1).unwrap().as_str()).join(",")
            )
        });

    let filename = format!(
        "{}.tex",
        args.input
    );
    std::fs::write(filename, latex_source.to_string())
        .with_context(|| "Failed to write out/out.tex")?;

    Ok(())
}
