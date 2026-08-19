use crate::html_markdown::light_box::LightBox;
use crate::html_markdown::link_card::LinkCard;
use crate::html_markdown::{collect_text, fence_for, find_first, find_first_with_class, get_attr, get_attr_u32, has_class, language_from_class};
use htmd::Element;
use htmd::element_handler::{HandlerResult, Handlers};
use markup5ever_rcdom::Node;
use std::rc::Rc;

pub struct HTMLHandlerExt;

/// Collect and normalize the text content of a node: collapse every
/// whitespace run (including the newlines/indentation of pretty-printed
/// HTML) into a single space.
fn text_of(node: &Rc<Node>) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}


impl HTMLHandlerExt {
    pub fn forum_lang_tag(_handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
        let code_node =
            find_first(element.node, "code").unwrap_or_else(|| element.node.clone());

        let lang = language_from_class(&code_node);

        let mut code_text = String::new();
        collect_text(&code_node, &mut code_text);
        let code_text = code_text.trim_end_matches('\n');

        let fence = fence_for(code_text);
        let content = format!("{fence}{lang}\n{code_text}\n{fence}\n");
        Some(HandlerResult::from(content))
    }

    pub fn light_box(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
        if !has_class(element.node, "lightbox-wrapper") {
            return handlers.fallback(element);
        }

        let img = find_first(element.node, "img")?;
        let src = get_attr(&img, "src").unwrap_or_default();
        let alt = get_attr(&img, "alt").unwrap_or_default();
        let width: u32 = get_attr_u32(&img, "width").unwrap_or(0);
        let height: u32 = get_attr_u32(&img, "height").unwrap_or(0);
        // let filename = text_of_first(element.node, "span", "filename").unwrap_or_default();
        // let size = text_of_first(element.node, "span", "informations").unwrap_or_default();

        let src = src.replace('"', "\\\"");
        let alt = alt.replace('"', "\\\"");


        let light_box_json = serde_json::to_string(&LightBox::new(src.into(), alt.into(), width, height)).unwrap_or_default();

        let content = format!(
            ":::forum-lightbox-wrapper\n{light_box_json}\n:::\n"
        );
        Some(HandlerResult::from(content))
    }

    pub fn link_card(handlers: &dyn Handlers, element: Element) -> Option<HandlerResult> {
        // Only oneboxes are handled here; every other <aside> (quotes, etc.)
        // falls through to the default rendering.
        if !has_class(element.node, "onebox") {
            return handlers.fallback(element);
        }

        let src = get_attr(element.node, "data-onebox-src").unwrap_or_default();

        let header = find_first_with_class(element.node, "header", "source")
            .and_then(|node| find_first(&node, "a"))
            .map(|node| text_of(&node))
            .unwrap_or_default();

        let thumbnail_src = find_first_with_class(element.node, "img", "thumbnail")
            .and_then(|img| get_attr(&img, "src"))
            .unwrap_or_default();

        let short_intro = find_first(element.node, "h3")
            .map(|node| text_of(&node))
            .unwrap_or_default();

        let card = if has_class(element.node, "githubrepo") {
            let gh_description =
                find_first_with_class(element.node, "span", "github-repo-description")
                    .map(|node| text_of(&node))
                    .unwrap_or_default();

            LinkCard::GHRepoCard {
                src,
                header,
                thumbnail_src,
                short_intro,
                gh_description,
            }
        } else {
            let header_img = find_first_with_class(element.node, "img", "site-icon")
                .and_then(|img| get_attr(&img, "src"))
                .unwrap_or_default();

            // Generic oneboxes carry their description in the first <p>.
            let gh_description = find_first(element.node, "p")
                .map(|node| text_of(&node))
                .unwrap_or_default();

            LinkCard::GenericLinkCard {
                src,
                header,
                header_img,
                thumbnail_src,
                short_intro,
                gh_description,
            }
        };

        let json = serde_json::to_string(&card).unwrap_or_default();
        let content = format!(":::link-card\n{json}\n:::\n");
        Some(HandlerResult::from(content))
    }
}