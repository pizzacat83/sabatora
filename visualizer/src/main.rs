use saba_core::renderer::css::parser::parse_css_stylesheet;
use saba_core::renderer::html::parser::HtmlParser;
use saba_core::renderer::html::token::HtmlTokenizer;
use saba_core::renderer::layout::box_tree::{
    construct_box_tree, BlockBoxChildren, InlineBox, InlineBoxData,
};
use saba_core::renderer::layout::box_tree::{BlockBox, BlockBoxData};
use saba_core::renderer::layout::layout_view::{get_style_content, LayoutView};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

const SAMPLE_HTML: &str = "<!doctype html><html><head></head><body><a>inline1 inline1 inline1</a>inline2 inline2 inline2<a>inline3 inline3 inline3</a><p>block4 block4 block4</p><p>block5 block5 block5</p>inline6 inline6 inline6</body></html>";

#[function_component(App)]
fn app() -> Html {
    let textarea_ref = use_node_ref();

    let html = use_state(|| SAMPLE_HTML.to_string());
    let box_tree = use_state(|| Some(construct_box_tree_from_html(SAMPLE_HTML.to_string())));

    let on_submit = {
        let html = html.clone();
        let box_tree = box_tree.clone();
        Callback::from(move |_| {
            box_tree.set(if html.is_empty() {
                None
            } else {
                Some(construct_box_tree_from_html(html.to_string()))
            });
        })
    };

    html! {
        <>
            <textarea style="width: 100%; height: 10rem;" ref={textarea_ref.clone()} value={html.to_string()} oninput={Callback::from(move |_| html.set(textarea_ref.cast::<HtmlTextAreaElement>().unwrap().value()))} />
            <button onclick={on_submit}>{"Visualize"}</button>
            {box_tree.as_ref().map(|box_tree| {
                html! {
                    <BlockBoxC block_box={box_tree.clone()} />
                }
            })}
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Properties, PartialEq)]
struct BlockBoxProps {
    block_box: BlockBox,
}

#[function_component(BlockBoxC)]
fn block_box(props: &BlockBoxProps) -> Html {
    let title = match props.block_box.data {
        BlockBoxData::Element(ref element) => format!("<{}>", element.tag_name()),
        BlockBoxData::Anonymous => "anonymous".to_string(),
    };

    let children = match props.block_box.children {
        BlockBoxChildren::Blocks(ref blocks) => blocks
            .iter()
            .map(|block| html! { <BlockBoxC block_box={block.clone()} /> })
            .collect::<Html>(),
        BlockBoxChildren::Inlines(ref inlines) => inlines
            .iter()
            .map(|inline| html! { <InlineBoxC inline_box={inline.clone()} /> })
            .collect::<Html>(),
        BlockBoxChildren::Empty => html! {},
    };

    let children_style = format!(
        "padding: 1rem; display: flex; gap: 1rem; {}",
        match props.block_box.children {
            BlockBoxChildren::Blocks(_) => "flex-direction: column;",
            BlockBoxChildren::Inlines(_) => "flex-direction: row;",
            BlockBoxChildren::Empty => "",
        }
    );

    html! {
        <div style="border: 1px solid black; text-wrap-mode: nowrap; min-width: fit-content;">
            <div style="border-bottom: 1px solid black; padding: 0.2rem 1rem; background-color: #eee;">{title}</div>
            <div style={children_style}>{children}</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct InlineBoxProps {
    inline_box: InlineBox,
}

#[function_component(InlineBoxC)]
fn inline_box(props: &InlineBoxProps) -> Html {
    let title = match props.inline_box.data {
        InlineBoxData::Element(ref element) => format!("<{}>", element.tag_name()),
        InlineBoxData::Anonymous => "anonymous".to_string(),
    };

    let children = html! {
        <>
            {props.inline_box.text.clone()}
            {
                props.inline_box.children.iter().map(|child| {
                    html! { <InlineBoxC inline_box={child.clone()} /> }
                }).collect::<Html>()
            }
        </>
    };

    html! {
        <div style="display: inline-block; border: 1px solid black; border-radius: 0.5rem;">
            <div style="border-bottom: 1px solid black; padding: 0.2rem 1rem; background-color: #eee; border-radius: 0.5rem 0.5rem 0 0;">{title}</div>
            <div style="padding: 1rem">{children}</div>
        </div>
    }
}

// TODO: use user-provided html
fn construct_box_tree_from_html(html: String) -> BlockBox {
    let t = HtmlTokenizer::new(html);
    let window = HtmlParser::new(t).construct_tree();
    let dom = window.borrow().document();
    let style = get_style_content(dom.clone());
    let cssom = parse_css_stylesheet(style);
    let layout_view = LayoutView::layout(dom, &cssom);
    construct_box_tree(layout_view)
}
