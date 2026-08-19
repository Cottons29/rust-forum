pub mod light_box;
mod handler_ext;
pub mod link_card;

use crate::html_markdown::handler_ext::HTMLHandlerExt;
use htmd::HtmlToMarkdown;
use markup5ever_rcdom::{Node, NodeData};
use std::rc::Rc;

const HTML_LITERIAL: &str = include_str!("../special_html/sample.json");

/// Recursively collect all text content under a node.
fn collect_text(node: &Rc<Node>, out: &mut String) {
    if let NodeData::Text { contents } = &node.data {
        out.push_str(&contents.borrow());
    }
    for child in node.children.borrow().iter() {
        collect_text(child, out);
    }
}

fn text_of_first(node: &Rc<Node>, tag: &str, class: &str) -> Option<String> {
    let el = find_first_with_class(node, tag, class)?;
    let mut text = String::new();
    collect_text(&el, &mut text);
    Some(text.trim().to_string())
}

fn find_first_with_class<'a>(node: &'a Rc<Node>, tag: &str, class: &str) -> Option<Rc<Node>> {
    if let NodeData::Element { name, .. } = &node.data {
        if &*name.local == tag && has_class(node, class) {
            return Some(node.clone());
        }
    }
    for child in node.children.borrow().iter() {
        if let Some(found) = find_first_with_class(child, tag, class) {
            return Some(found);
        }
    }
    None
}

/// Find the first descendant element with the given tag name, e.g. "code".
fn find_first(node: &Rc<Node>, tag: &str) -> Option<Rc<Node>> {
    if let NodeData::Element { name, .. } = &node.data {
        if &*name.local == tag {
            return Some(node.clone());
        }
    }
    for child in node.children.borrow().iter() {
        if let Some(found) = find_first(child, tag) {
            return Some(found);
        }
    }
    None
}

fn language_from_class(node: &Rc<Node>) -> String {
    if let NodeData::Element { attrs, .. } = &node.data {
        for attr in attrs.borrow().iter() {
            if &*attr.name.local == "class" {
                if let Some(lang) = attr
                    .value
                    .split_whitespace()
                    .find(|c| c.starts_with("language-") || c.starts_with("lang-"))
                {
                    return lang
                        .trim_start_matches("language-")
                        .trim_start_matches("lang-")
                        .to_string();
                }
            }
        }
    }
    String::new()
}

fn normalize_code_fences(markdown: &str) -> String {
    let mut result = String::new();
    let mut in_fence = false;
    let mut fence_char = ' ';

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];

        if !in_fence {
            if let Some(info) = trimmed.strip_prefix("~~~") {
                // opening a ~~~ fence -> rewrite as ```
                result.push_str(indent);
                result.push_str("```");
                result.push_str(info.trim_end());
                result.push('\n');
                in_fence = true;
                fence_char = '~';
                continue;
            } else if trimmed.starts_with("```") {
                in_fence = true;
                fence_char = '`';
            }
        } else if fence_char == '~' && trimmed.starts_with("~~~") {
            // closing ~~~ fence -> rewrite as ```
            result.push_str(indent);
            result.push_str("```");
            result.push('\n');
            in_fence = false;
            continue;
        } else if fence_char == '`' && trimmed.starts_with("```") {
            in_fence = false;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Find the length of the longest run of consecutive backticks in `text`.
fn max_backtick_run(text: &str) -> usize {
    let mut max_run = 0;
    let mut current = 0;
    for c in text.chars() {
        if c == '`' {
            current += 1;
            max_run = max_run.max(current);
        } else {
            current = 0;
        }
    }
    max_run
}

fn get_attr(node: &Rc<Node>, attr_name: &str) -> Option<String> {
    if let NodeData::Element { attrs, .. } = &node.data {
        for attr in attrs.borrow().iter() {
            if &*attr.name.local == attr_name {
                return Some(attr.value.to_string());
            }
        }
    }
    None
}

fn fence_for(text: &str) -> String {
    let longest = max_backtick_run(text);
    let len = (longest + 1).max(3);
    "`".repeat(len)
}

fn has_class(node: &Rc<Node>, class: &str) -> bool {
    get_attr(node, "class")
        .map(|c| c.split_whitespace().any(|c| c == class))
        .unwrap_or(false)
}

fn get_attr_u32(node: &Rc<Node>, attr_name: &str) -> Option<u32> {
    get_attr(node, attr_name)?.parse::<u32>().ok()
}

fn build_converter() -> HtmlToMarkdown {
    HtmlToMarkdown::builder()
        .add_handler(vec!["pre"], HTMLHandlerExt::forum_lang_tag)
        .add_handler(vec!["div"], HTMLHandlerExt::light_box)
        .add_handler(vec!["aside"], HTMLHandlerExt::link_card)
        .build()
}

pub fn html_to_markdown(input: &str) -> Rc<str> {
    let md = build_converter().convert(input).unwrap_or_default();
    let normalized = normalize_code_fences(&md);
    Rc::from(normalized)
}

#[cfg(test)]
mod test {
    use crate::html_markdown::{
        HTML_LITERIAL, build_converter, collect_text, find_first, language_from_class,
        normalize_code_fences,
    };
    use crate::models::posts::PostRoot;
    use htmd::element_handler::{HandlerResult, Handlers};
    use htmd::{Element, HtmlToMarkdown};

    #[test]
    fn test_json() {
        let post_stream = serde_json::from_str::<PostRoot>(HTML_LITERIAL);

        assert!(post_stream.is_ok());
    }

    #[test]
    fn test_real_forum_fmt() {
        const POST: &str = r#"<p>To format code in this forum you need to surround the code with three backticks (<code>```</code>). For example,
    typing this...</p>\n
<pre><code>```\nfn main() {\n    println!()\n}\n```\n</code></pre>\n<p>...will be rendered as highlighted Rust code,
    like so:</p>\n
<pre data-code-wrap=\"rust\"><code class=\"lang-rust\">fn main() {\n    println!()\n}\n</code></pre>\n<p>You can also
    tell the forum software which language you're using by adding the language after the opening backticks.</p>\n
<pre><code>```cpp\n#include &lt;iostream&gt;\nint main() {\n  std::cout &lt;&lt; \"Hello World!\";\n}\n```\n</code></pre>\n
<p>Will appear with highlighting as:</p>\n
<pre data-code-wrap=\"cpp\"><code class=\"lang-cpp\">#include &lt;iostream&gt;\nint main() {\n  std::cout &lt;&lt; \"Hello World!\";\n}\n</code></pre>\n
<p>The currently supported languages include: <code>apache</code>, <code>bash</code>, <code>cs</code>, <code>cpp</code>,
    <code>css</code>, <code>coffeescript</code>, <code>diff</code>, <code>xml</code>, <code>http</code>,
    <code>ini</code>, <code>json</code>, <code>java</code>, <code>javascript</code>, <code>makefile</code>, <code>markdown</code>,
    <code>nginx</code>, <code>objectivec</code>, <code>ruby</code>, <code>perl</code>, <code>php</code>,
    <code>python</code>, <code>sql</code>, <code>handlebars</code>, <code>rust</code> and <code>dust</code>. Others may
    be added in the future.</p>\n<p>It's also possible to add plain text by using <code>text</code> as the language:
</p>\n
<pre><code
        class=\"lang-plaintext\">```text\nError 365: Invalid command.\nSee help for a list of valid commands.\n```\n</code></pre>\n
<p>This will be rendered without any highlighting:</p>\n
<pre><code class=\"lang-plaintext\">Error 365: Invalid command.\nSee help for a list of valid commands.\n</code></pre>\n
<p>If your keyboard layout does not have the <kbd>`</kbd> key, you can also use a <kbd>~</kbd>.</p>\n
<pre><code>~~~\nfn main() {\n    println!()\n}\n~~~\n</code></pre>\n<p>...will be rendered as highlighted Rust code,
    like so:</p>\n
<pre data-code-wrap=\"rust\"><code class=\"lang-rust\">fn main() {\n    println!()\n}\n</code></pre>"#;

        let md = build_converter().convert(POST).unwrap();
        println!("{}", md);
    }

    #[test]
    fn test_html_markdown() {
        let post_stream = serde_json::from_str::<PostRoot>(HTML_LITERIAL);

        assert!(post_stream.is_ok());

        let post = post_stream
            .as_ref()
            .unwrap()
            .post_stream
            .posts
            .iter()
            .find(|item| item.id == 165427);

        assert!(post.is_some());

        let markdown = htmd::convert(&post.unwrap().cooked);

        assert!(markdown.is_ok());

        eprintln!("{}", markdown.unwrap());

        let converter = HtmlToMarkdown::builder()
            .add_handler(vec!["pre"], |_handlers: &dyn Handlers, element: Element| {
                let code_node =
                    find_first(element.node, "code").unwrap_or_else(|| element.node.clone());

                let lang = language_from_class(&code_node);

                let mut code_text = String::new();
                collect_text(&code_node, &mut code_text);

                let content = format!("```{}\n{}\n```\n", lang, code_text.trim_end());
                Some(HandlerResult::from(content))
            })
            .build();

        let md = converter.convert(&post.unwrap().cooked).unwrap();
        println!("{}", md);

        let string = "~~~rust\nfn main() {}\n~~~\n";

        let normalized = normalize_code_fences(&string);

        println!("Normalized : {}", normalized);
    }

    #[test]
    fn test_parse_html_block() {
        const HTML: &str = include_str!("../sample_html.html");

        let item = build_converter().convert(HTML).unwrap();

        eprintln!("{}", item);
    }

    #[test]
    fn test_one_box() {

    }

    #[test]
    fn test_link_card() {
        for (name, html) in [
            ("generic", include_str!("../special_html/141914.html")),
            ("github", include_str!("../special_html/141777.html")),
        ] {
            eprintln!("===== {name} =====");
            let md = build_converter().convert(html).unwrap();
            eprintln!("{md}");
        }
    }
}
