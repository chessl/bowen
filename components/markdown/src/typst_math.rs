use std::ops::Range;

use errors::{Result, bail};
use once_cell::sync::Lazy;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_escape::escape_html;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Feature, Features, Library, LibraryExt, World};
use typst_html::{HtmlDocument, HtmlElement, HtmlNode};
use typst_kit::fonts::FontStore;

pub const TYPST_MATH_PLACEHOLDER: &str = "@@ZOLA_TYPST_MATH_PLACEHOLDER@@";

static TYPST_LIBRARY: Lazy<LazyHash<Library>> = Lazy::new(|| {
    let features: Features = std::iter::once(Feature::Html).collect();
    LazyHash::new(Library::builder().with_features(features).build())
});

static TYPST_FONTS: Lazy<FontStore> = Lazy::new(|| {
    let mut store = FontStore::new();
    store.extend(typst_kit::fonts::embedded());
    store
});

#[derive(Debug)]
pub struct TypstMath {
    pub span: Range<usize>,
    pub html: String,
    pub block: bool,
}

struct TypstMathWorld {
    main: FileId,
    source: Source,
}

impl TypstMathWorld {
    fn new(source: String) -> Self {
        let main =
            RootedPath::new(VirtualRoot::Project, VirtualPath::new("main.typ").unwrap()).intern();
        let source = Source::new(main, source);
        Self { main, source }
    }
}

impl World for TypstMathWorld {
    fn library(&self) -> &LazyHash<Library> {
        &TYPST_LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        TYPST_FONTS.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main { Ok(self.source.clone()) } else { Err(FileError::AccessDenied) }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            Ok(Bytes::from_string(self.source.text().to_owned()))
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        TYPST_FONTS.font(index)
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

pub fn extract(content: &str, opts: Options) -> Result<(String, Vec<TypstMath>)> {
    let skip_ranges = ranges_to_skip(content, opts);
    let mut skip_idx = 0;
    let mut output = String::with_capacity(content.len());
    let mut math = Vec::new();
    let mut idx = 0;

    while idx < content.len() {
        while skip_idx < skip_ranges.len() && skip_ranges[skip_idx].end <= idx {
            skip_idx += 1;
        }

        if let Some(range) = skip_ranges.get(skip_idx)
            && range.start <= idx
            && idx < range.end
        {
            output.push_str(&content[idx..range.end]);
            idx = range.end;
            continue;
        }

        let bytes = content.as_bytes();
        if bytes[idx] == b'$' && !is_escaped(content, idx) {
            let is_block = idx + 1 < content.len() && bytes[idx + 1] == b'$';
            let delimiter_len = if is_block { 2 } else { 1 };
            let closing = if is_block {
                find_closing_double_dollar(content, idx + delimiter_len)
            } else {
                find_closing_single_dollar(content, idx + delimiter_len)
            };

            if let Some(closing_start) = closing {
                let expression = &content[idx + delimiter_len..closing_start];
                if !expression.trim().is_empty() {
                    let html = render_typst_math(expression, is_block)?;
                    let start = output.len();
                    output.push_str(TYPST_MATH_PLACEHOLDER);
                    let end = output.len();
                    math.push(TypstMath { span: start..end, html, block: is_block });
                    idx = closing_start + delimiter_len;
                    continue;
                }
            }
        }

        let ch = content[idx..].chars().next().unwrap();
        output.push(ch);
        idx += ch.len_utf8();
    }

    Ok((output, math))
}

fn ranges_to_skip(content: &str, opts: Options) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut code_block_start = None;

    for (event, range) in Parser::new_ext(content, opts).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => code_block_start = Some(range.start),
            Event::End(TagEnd::CodeBlock) => {
                if let Some(start) = code_block_start.take() {
                    ranges.push(start..range.end);
                }
            }
            Event::Code(_) | Event::Html(_) | Event::InlineHtml(_) => ranges.push(range),
            _ => {}
        }
    }

    if let Some(start) = code_block_start {
        ranges.push(start..content.len());
    }

    merge_ranges(ranges)
}

fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();

    for range in ranges {
        if range.is_empty() {
            continue;
        }

        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
            continue;
        }

        merged.push(range);
    }

    merged
}

fn find_closing_double_dollar(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut idx = start;
    while idx + 1 < content.len() {
        if bytes[idx] == b'$' && bytes[idx + 1] == b'$' && !is_escaped(content, idx) {
            return Some(idx);
        }
        idx += content[idx..].chars().next().unwrap().len_utf8();
    }
    None
}

fn find_closing_single_dollar(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut idx = start;
    while idx < content.len() {
        if bytes[idx] == b'\n' {
            return None;
        }

        if bytes[idx] == b'$'
            && !is_escaped(content, idx)
            && (idx + 1 == content.len() || bytes[idx + 1] != b'$')
        {
            return Some(idx);
        }
        idx += content[idx..].chars().next().unwrap().len_utf8();
    }
    None
}

fn is_escaped(content: &str, idx: usize) -> bool {
    let mut backslashes = 0;
    let mut cursor = idx;
    let bytes = content.as_bytes();

    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }

    backslashes % 2 == 1
}

fn render_typst_math(expression: &str, block: bool) -> Result<String> {
    let expression = expression.trim();
    let source = if block { format!("$ {expression} $") } else { format!("${expression}$") };
    let world = TypstMathWorld::new(source);
    let document = typst::compile::<HtmlDocument>(&world)
        .output
        .map_err(|diagnostics| errors::Error::msg(format_diagnostics(&diagnostics)))?;
    let math = find_math_element(document.root())
        .ok_or_else(|| errors::Error::msg("Typst math did not produce a MathML element"))?;

    let mut html = String::new();
    write_html_element(&mut html, math)?;
    Ok(html)
}

fn find_math_element(element: &HtmlElement) -> Option<&HtmlElement> {
    if element.tag == typst_html::tag::mathml::math {
        return Some(element);
    }

    element.children.iter().find_map(|node| match node {
        HtmlNode::Element(child) => find_math_element(child),
        _ => None,
    })
}

fn write_html_element(output: &mut String, element: &HtmlElement) -> Result<()> {
    let tag = element.tag.resolve();
    output.push('<');
    output.push_str(tag.as_str());

    for (attr, value) in &element.attrs.0 {
        output.push(' ');
        output.push_str(attr.resolve().as_str());
        if !value.is_empty() {
            output.push_str("=\"");
            escape_html(&mut *output, value.as_str())?;
            output.push('"');
        }
    }

    output.push('>');
    for child in &element.children {
        write_html_node(output, child)?;
    }
    output.push_str("</");
    output.push_str(tag.as_str());
    output.push('>');
    Ok(())
}

fn write_html_node(output: &mut String, node: &HtmlNode) -> Result<()> {
    match node {
        HtmlNode::Tag(_) => {}
        HtmlNode::Text(text, _) => escape_html(&mut *output, text.as_str())?,
        HtmlNode::Element(element) => write_html_element(output, element)?,
        HtmlNode::Frame(_) => bail!("Typst math rendered a frame instead of MathML"),
    }
    Ok(())
}

fn format_diagnostics(diagnostics: &[SourceDiagnostic]) -> String {
    let mut messages = Vec::new();

    for diagnostic in diagnostics {
        let mut message = diagnostic.message.to_string();
        for hint in &diagnostic.hints {
            message.push_str("; ");
            message.push_str(hint.v.as_str());
        }
        messages.push(message);
    }

    if messages.is_empty() {
        "Typst math compilation failed".to_string()
    } else {
        format!("Typst math compilation failed: {}", messages.join("; "))
    }
}
