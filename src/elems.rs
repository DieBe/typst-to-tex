use typst::foundations::Content;
use typst::model::{EnumItem, ListItem, TermItem};
use typst::{
    foundations::{ContextElem, SequenceElem, StyledElem, SymbolElem, TargetElem},
    html::{FrameElem, HtmlElem},
    introspection::{CounterDisplayElem, MetadataElem, TagElem},
    layout::{
        AlignElem, BlockElem, BoxElem, ColbreakElem, ColumnsElem, FlushElem, GridCell, GridElem,
        GridFooter, GridHLine, GridHeader, GridVLine, HElem, HideElem, InlineElem, MoveElem,
        PadElem, PageElem, PagebreakElem, PlaceElem, RepeatElem, RotateElem, ScaleElem, SkewElem,
        StackElem, VElem,
    },
    math::{
        AccentElem, AlignPointElem, AttachElem, BinomElem, CancelElem, CasesElem, ClassElem,
        EquationElem, FracElem, LimitsElem, LrElem, MatElem, MidElem, OpElem, OverbraceElem,
        OverbracketElem, OverlineElem, OverparenElem, OvershellElem, PrimesElem, RootElem,
        ScriptsElem, StretchElem, UnderbraceElem, UnderbracketElem, UnderlineElem, UnderparenElem,
        UndershellElem, VecElem,
    },
    model::{
        BibliographyElem, CiteElem, CiteGroup, DocumentElem, EmphElem, EnumElem, FigureCaption,
        FigureElem, FootnoteElem, FootnoteEntry, HeadingElem, LinkElem, ListElem, OutlineElem,
        OutlineEntry, ParElem, ParLine, ParLineMarker, ParbreakElem, QuoteElem, RefElem,
        StrongElem, TableCell, TableElem, TableFooter, TableHLine, TableHeader, TableVLine,
        TermsElem,
    },
    pdf::EmbedElem,
    text::{
        HighlightElem, LinebreakElem, RawElem, RawLine, SmallcapsElem, SmartQuoteElem, SpaceElem,
        StrikeElem, SubElem, SuperElem, TextElem,
    },
    visualize::{
        CircleElem, CurveClose, CurveCubic, CurveElem, CurveLine, CurveMove, CurveQuad,
        EllipseElem, ImageElem, LineElem, PathElem, PolygonElem, RectElem, SquareElem,
    },
};

macro_rules! define_typed{
    (enum $enum_name:ident {
        $($name:ident($native:path$(;<$vallife:lifetime>)?)),*$(,)?
    };
        
    $(unused ($why:literal) {$($name_:ident($native_:path$(;<$vallife_:lifetime>)?)),*$(,)?})+

    fn $from_name:ident) => {
        #[allow(dead_code)]
        pub enum $enum_name {
            $(
                $name($native)
            ),*,
            Ignored,
        }

        pub fn $from_name<'a>(mut content: Content) -> Elem {
            $(
                content = match content.into_packed::<$native>() {
                    Ok(val) => return $enum_name::$name(val.unpack()),
                    Err(content) => content,
                };
            )*
            $(
                $(
                    match content.to_packed::<$native_>() {
                        Some(_) => {
                            println!("Ignoring {} because {}", stringify!($name_), $why);
                            return $enum_name::Ignored;
                        },
                        None => {},
                    };
                )*
            )*
            panic!("Did not manage to convert {content:?} into a native value")
        }
    }
}

define_typed! {
enum Elem {
    HideElem(HideElem),
    CiteElem(CiteElem),
    CiteGroup(CiteGroup),
    EmphElem(EmphElem),
    EnumElem(EnumElem),
    FigureElem(FigureElem),
    FootnoteElem(FootnoteElem),
    HeadingElem(HeadingElem),
    LinkElem(LinkElem),
    ListElem(ListElem),
    ParElem(ParElem),
    ParLineMarker(ParLineMarker),
    ParbreakElem(ParbreakElem),
    QuoteElem(QuoteElem),
    RefElem(RefElem),
    StrongElem(StrongElem),
    HighlightElem(HighlightElem),
    LinebreakElem(LinebreakElem),
    TableElem(TableElem),
    RawElem(RawElem),
    SmallcapsElem(SmallcapsElem),
    SmartQuoteElem(SmartQuoteElem),
    SpaceElem(SpaceElem),
    StrikeElem(StrikeElem),
    SubElem(SubElem),
    SuperElem(SuperElem),
    TextElem(TextElem),
    UnderlineElem(UnderlineElem),
    ContextElem(ContextElem),
    SequenceElem(SequenceElem),
    StyledElem(StyledElem),
    SymbolElem(SymbolElem),
    EquationElem(EquationElem),
    BoxElem(BoxElem),
};

unused ("Inner element") {
    EnumItem(EnumItem;<'a>),
    RawLine(RawLine),
    ListItem(ListItem;<'a>),
    FootnoteEntry(FootnoteEntry),
    BibliographyElem(BibliographyElem),
}


unused ("outline") {
    OutlineElem(OutlineElem),
    OutlineEntry(OutlineEntry),
}

unused ("Table internal") {
    TableCell(TableCell),
    TableFooter(TableFooter),
    TableHLine(TableHLine),
    TableHeader(TableHeader),
    TableVLine(TableVLine),
}

unused ("Term stuff") {
    TermItem(TermItem;<'a>),
    TermsElem(TermsElem),
}

unused ("Just hard to handle") {
    EmbedElem(EmbedElem),
    
}

unused ("Math mode") {
    RootElem(RootElem),
    ScriptsElem(ScriptsElem),
    StretchElem(StretchElem),
    MathOverlineElem(typst::math::OverlineElem),
    RotateElem(RotateElem),
    ScaleElem(ScaleElem),
    SkewElem(SkewElem),
    StackElem(StackElem),
    VElem(VElem),
    AccentElem(AccentElem),
    AlignPointElem(AlignPointElem),
    AttachElem(AttachElem),
    BinomElem(BinomElem),
    CancelElem(CancelElem),
    CasesElem(CasesElem),
    ClassElem(ClassElem),
    FracElem(FracElem),
    LimitsElem(LimitsElem),
    LrElem(LrElem),
    MatElem(MatElem),
    MidElem(MidElem),
    OpElem(OpElem),
    OverbraceElem(OverbraceElem),
    OverbracketElem(OverbracketElem),
    OverlineElem(OverlineElem),
    OverparenElem(OverparenElem),
    OvershellElem(OvershellElem),
    PrimesElem(PrimesElem),
    UnderbraceElem(UnderbraceElem),
    UnderbracketElem(UnderbracketElem),
    MathUnderlineElem(typst::math::UnderlineElem),
    UnderparenElem(UnderparenElem),
    UndershellElem(UndershellElem),
    VecElem(VecElem),
    MoveElem(MoveElem),
    HElem(HElem),
    PadElem(PadElem),
    PlaceElem(PlaceElem),
    RepeatElem(RepeatElem),
}

unused ("Internal") {
    TargetElem(TargetElem),
    ParLine(ParLine),
}

unused ("Layout stuff") {
    FigureCaption(FigureCaption),
    DocumentElem(DocumentElem),
    PagebreakElem(PagebreakElem),
    InlineElem(InlineElem),
    PageElem(PageElem),
    BlockElem(BlockElem),
    ColbreakElem(ColbreakElem),
    ColumnsElem(ColumnsElem),
    FlushElem(FlushElem),
    SquareElem(SquareElem),
    CircleElem(CircleElem),
    CurveClose(CurveClose),
    CurveCubic(CurveCubic),
    CurveElem(CurveElem),
    CurveLine(CurveLine),
    CurveMove(CurveMove),
    CurveQuad(CurveQuad),
    EllipseElem(EllipseElem),
    ImageElem(ImageElem),
    LineElem(LineElem),
    PathElem(PathElem),
    PolygonElem(PolygonElem),
    RectElem(RectElem),
}

unused("HTML stuff") {
    FrameElem(FrameElem),
    HtmlElem(HtmlElem),
    CounterDisplayElem(CounterDisplayElem),
    MetadataElem(MetadataElem),
    TagElem(TagElem),
    AlignElem(AlignElem),
}

unused("Grid should be handled manually") {
    GridCell(GridCell),
    GridElem(GridElem),
    GridFooter(GridFooter),
    GridHLine(GridHLine),
    GridHeader(GridHeader),
    GridVLine(GridVLine),
}

fn from_native
}
