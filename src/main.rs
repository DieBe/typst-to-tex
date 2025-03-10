mod elems;

use clap::Parser;
use elems::from_native;
use itertools::Itertools;
// use ecow::{eco_format, EcoString};
use tinymist_project::CompileOnceArgs;
use tinymist_project::WorldProvider;

use color_eyre::{eyre::Context, Result};
use typst::comemo::Track;
use typst::engine::Route;
use typst::engine::Sink;
use typst::engine::Traced;
use typst::foundations::Content;
use typst::foundations::StyleChain;
use typst::foundations::Styles;
use typst::syntax::Span;
use typst::World;
use typst::ROUTINES;

/// Common arguments of compile, watch, and query.
#[derive(Debug, Clone, Parser, Default)]
pub struct CompileArgs {
    #[clap(flatten)]
    pub compile: CompileOnceArgs,
}

fn eval_typst(world: &dyn World) -> Result<Content> {
    let library = world.library();

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

pub enum TexBlock {
    String(String),
    Label(String),
    Ref(String),
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
            TexBlock::Ref(l) => {
                let sup = match l.split(":").next() {
                    None => "",
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

fn into_latex(content: Content, sc: StyleChain) -> TexBlock {
    let label = content.label();
    let native = from_native(content);

    match native {
        elems::Elem::HideElem(_) => TexBlock::Nothing,
        elems::Elem::CiteElem(_) | elems::Elem::CiteGroup(_) => todo!("Citations"),
        elems::Elem::EmphElem(emph_elem) => TexBlock::String(format!(
            "\\textit{{{}}}",
            into_latex(emph_elem.body, sc).emit()
        )),
        elems::Elem::EnumElem(_enum_elem) => todo!("Support enumerate"),
        elems::Elem::FigureElem(_figure_elem) => todo!("Support figures"),
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
                into_latex(heading_elem.body, sc).emit()
            ));
            let l = label
                .map(|l| l.resolve().to_string())
                .map(TexBlock::Label)
                .map(Box::new);

            TexBlock::Seq(vec![heading, TexBlock::Maybe(l)])
        }
        elems::Elem::LinkElem(_link_elem) => todo!(),
        elems::Elem::ListElem(_list_elem) => todo!(),
        elems::Elem::ParElem(par_elem) => into_latex(par_elem.body, sc),
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
        elems::Elem::SequenceElem(s) => TexBlock::Seq(
            s.children
                .into_iter()
                .map(|content| into_latex(content, sc))
                .collect(),
        ),
        elems::Elem::StyledElem(styled_elem) => into_latex(styled_elem.child, sc),
        elems::Elem::SymbolElem(_symbol_elem) => todo!(),
        elems::Elem::Ignored => todo!(),
    }
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = CompileArgs::parse();

    let universe = args
        .compile
        .resolve()
        .with_context(|| "failed to call resolve")?;
    let world = universe.snapshot();

    let mut content = eval_typst(&world)?;

    println!("{content:#?}");

    let styles = Styles::new();
    let sc = StyleChain::new(&styles);

    content.materialize(sc);

    let latex = into_latex(content, sc);

    let latex_source = format!("\\documentclass{{article}}\\begin{{document}}{}\\end{{document}}", latex.emit());
    println!("\n\n{}", latex_source);

    std::fs::create_dir_all("out").with_context(|| "Failed to create dir out")?;
    std::fs::write("out/out.tex", latex_source).with_context(|| "Failed to write out/out.tex")?;

    Ok(())
}
