mod elems;

use camino::Utf8Path;
use clap::Parser;
use elems::from_native;
use itertools::Itertools;
// use ecow::{eco_format, EcoString};
use tinymist_project::CompileOnceArgs;
use tinymist_project::WorldProvider;

use color_eyre::{eyre::Context, Result};
use typst::comemo::Track;
use typst::engine::Engine;
use typst::engine::Route;
use typst::engine::Sink;
use typst::engine::Traced;
use typst::foundations::Content;
use typst::foundations::Smart;
use typst::foundations::StyleChain;
use typst::foundations::Styles;
use typst::introspection::Introspector;
use typst::layout::Margin;
use typst::layout::PageElem;
use typst::layout::Rel;
use typst::syntax::Span;
use typst::text::BottomEdge;
use typst::text::BottomEdgeMetric;
use typst::text::TextElem;
use typst::text::TopEdge;
use typst::text::TopEdgeMetric;
use typst::Library;
use typst::World;
use typst::ROUTINES;
use typst_layout::layout_document;
use typst_pdf::pdf;
use typst_pdf::PdfOptions;

/// Common arguments of compile, watch, and query.
#[derive(Debug, Clone, Parser, Default)]
pub struct CompileArgs {
    #[clap(flatten)]
    pub compile: CompileOnceArgs,
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

fn compile_subcontent(
    world: &dyn World,
    mut inner_content: Content,
    sc: StyleChain,
    output_file: &Utf8Path,
) -> Result<()> {
    let empty_introspector = Introspector::default();
    let traced = Traced::new(Span::detached());
    let mut sink = Sink::new();
    let mut engine = Engine {
        routines: &ROUTINES,
        world: world.track(),
        introspector: empty_introspector.track(),
        traced: traced.track(),
        sink: sink.track_mut(),
        route: Route::default(),
    };

    // let page = PageElem::new();
    let styles = &[
        // PageElem::set_width(Smart::Auto).wrap(),
        // PageElem::set_height(Smart::Auto).wrap(),
        // PageElem::set_margin(Margin::splat(Some(Smart::Custom(Rel::zero())))).wrap(),
        // TextElem::set_top_edge(TopEdge::Metric(TopEdgeMetric::Ascender)).wrap(),
        // TextElem::set_bottom_edge(BottomEdge::Metric(BottomEdgeMetric::Descender)).wrap(),
    ];
    let sc = sc.chain(styles);
    // let content = Content::new(page);
    inner_content.materialize(sc);

    for _ in 1..5 {
        layout_document(&mut engine, &mut inner_content, sc).unwrap();
    }
    let doc = layout_document(&mut engine, &mut inner_content, sc).unwrap();

    let pdf = pdf(&doc, &PdfOptions::default()).unwrap();

    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create dir {parent}"))?;
    }

    std::fs::write(output_file, pdf)
        .with_context(|| format!("Failed to write pdf to {output_file}"))?;

    Ok(())
}

pub enum TexBlock {
    String(String),
    Label(String),
    Ref(String),
    Figure {
        content_placeholder: String,
        caption: Option<Box<TexBlock>>,
    },
    Seq(Vec<TexBlock>),
    Maybe(Option<Box<TexBlock>>),
    Nothing,
}

impl TexBlock {
    fn emit(&self) -> String {
        match self {
            TexBlock::String(s) => s.clone(),
            TexBlock::Label(l) => {
                format!("\\label{{{l}}}")
            }
            TexBlock::Figure {
                content_placeholder,
                caption,
            } => {
                indoc::formatdoc!(
                    r#"
                    \begin{{figure}}
                        \includegraphics{{../generated/test.pdf}}
                        {caption}
                    \end{{figure}}
                    "#,
                    caption = if let Some(caption) = caption.as_ref().map(|s| s.emit()) {
                        format!(r"\caption{{{caption}}}")
                    } else {
                        String::new()
                    }
                )
            }
            TexBlock::Ref(l) => {
                let sup = match l.split(":").next() {
                    None => "",
                    Some("fig") => "Fig.",
                    Some("sec") => "Section",
                    other => {
                        println!("Unknown label supplement: {other:?}");
                        ""
                    }
                };

                format!("{sup} \\ref{{{l}}}")
            }
            TexBlock::Seq(inner) => inner.iter().map(|i| i.emit()).join(""),
            TexBlock::Maybe(inner) => inner
                .as_ref()
                .map(|inner| inner.emit())
                .unwrap_or_else(|| String::new()),
            TexBlock::Nothing => String::new(),
        }
    }
}

fn into_latex(mut content: Content, sc: StyleChain, world: &dyn World) -> Result<TexBlock> {
    content.materialize(sc);

    let label = TexBlock::Maybe(
        content
            .label()
            .map(|l| l.resolve().to_string())
            .map(TexBlock::Label)
            .map(Box::new),
    );
    let native = from_native(content);

    let result = match native {
        elems::Elem::HideElem(_) => TexBlock::Nothing,
        elems::Elem::CiteElem(_) | elems::Elem::CiteGroup(_) => todo!("Citations"),
        elems::Elem::EmphElem(emph_elem) => TexBlock::String(format!(
            "\\textit{{{}}}",
            into_latex(emph_elem.body, sc, world)?.emit()
        )),
        elems::Elem::EnumElem(_enum_elem) => todo!("Support enumerate"),
        elems::Elem::FigureElem(figure_elem) => {
            let caption = figure_elem
                .caption(sc)
                .as_ref()
                .map(|cap| into_latex(cap.body.clone(), sc, world))
                .transpose()?
                .map(|cap| TexBlock::Seq(vec![cap, label]))
                .map(Box::new);

            compile_subcontent(world, figure_elem.body, sc, "generated/test.pdf".into())?;

            TexBlock::Figure {
                content_placeholder: { "Placeholder".to_string() },
                caption,
            }
        }
        elems::Elem::FootnoteElem(_footnote_elem) => {
            todo!("Support footnotes")
        }
        elems::Elem::HeadingElem(heading_elem) => {
            let level = heading_elem.offset(sc);

            let h = match level {
                0 => "section",
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
                into_latex(heading_elem.body, sc, world)?.emit()
            ));

            TexBlock::Seq(vec![heading, label])
        }
        elems::Elem::LinkElem(_link_elem) => todo!(),
        elems::Elem::ListElem(_list_elem) => todo!(),
        elems::Elem::ParElem(par_elem) => into_latex(par_elem.body, sc, world)?,
        elems::Elem::ParLineMarker(_) => TexBlock::Nothing,
        elems::Elem::ParbreakElem(_) => TexBlock::String("\n\n".to_string()),
        elems::Elem::QuoteElem(_quote_elem) => todo!(),
        elems::Elem::RefElem(ref_elem) => {
            println!("Ref had supplement {:?}", ref_elem.supplement(sc));
            TexBlock::Ref(ref_elem.target.resolve().to_string())
        }
        elems::Elem::StrongElem(_strong_elem) => todo!(),
        elems::Elem::HighlightElem(_highlight_elem) => todo!(),
        elems::Elem::LinebreakElem(_linebreak_elem) => todo!(),
        elems::Elem::TableElem(_table_elem) => todo!(),
        elems::Elem::RawElem(_raw_elem) => todo!(),
        elems::Elem::SmallcapsElem(_smallcaps_elem) => todo!(),
        elems::Elem::SmartQuoteElem(_smart_quote_elem) => todo!(),
        elems::Elem::SpaceElem(_) => TexBlock::String(" ".to_string()),
        elems::Elem::StrikeElem(_strike_elem) => todo!(),
        elems::Elem::SubElem(_sub_elem) => todo!(),
        elems::Elem::SuperElem(_super_elem) => todo!(),
        elems::Elem::TextElem(text) => TexBlock::String(text.text.to_string()),
        elems::Elem::UnderlineElem(_underline_elem) => todo!(),
        elems::Elem::ContextElem(_context_elem) => todo!(),
        elems::Elem::SequenceElem(s) => {
            println!("New seq: ");
            for elem in &s.children {
                println!("  {elem:?}")
            }

            TexBlock::Seq(
                s.children
                    .iter()
                    .map(|content| into_latex(content.clone(), sc, world))
                    .collect::<Result<_>>()?,
            )
        }
        elems::Elem::StyledElem(styled_elem) => {
            let sc = sc.chain(styled_elem.styles.as_slice());
            into_latex(styled_elem.child, sc, world)?
        }
        elems::Elem::SymbolElem(_symbol_elem) => todo!(),
        elems::Elem::Ignored => todo!(),
    };
    Ok(result)
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = CompileArgs::parse();

    let universe = args
        .compile
        .resolve()
        .with_context(|| "failed to call resolve")?;
    let mut world = universe.snapshot();

    let content = eval_typst(&world)?;

    let library = Library::builder().build();
    let sc = StyleChain::new(&library.styles);

    // compile_subcontent(&mut world, content, sc, "generated/test.pdf".into())?;

    let latex = into_latex(content, sc, &world)?;

    let latex_source = indoc::formatdoc!(
        r#"\documentclass{{article}}
        \usepackage{{graphicx}}
        \begin{{document}}
        {}
        \end{{document}}
        "#,
        latex.emit()
    );
    println!("\n\n{}", latex_source);

    std::fs::create_dir_all("out").with_context(|| "Failed to create dir out")?;
    std::fs::write("out/out.tex", latex_source).with_context(|| "Failed to write out/out.tex")?;

    Ok(())
}
