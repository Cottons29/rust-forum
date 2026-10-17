use crate::components::side_bar::category_row::CategoryRow;
use crate::components::side_bar::skeleton_category_row::SkeletonCategoryRow;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::console::info;
use aimer::share::RcProjectExt;
use aimer::style::{LayoutSpacing, TextStyle, Theme};
use aimer::{
    AnyWidget, AsyncBuilder, AsyncSnapshot, BuildContext, CollapsibleList, Column, FontWeight,
    ListBody, ListHeader, ProviderHandle, StatelessWidget, Text, Widget,
};

#[derive(StatelessWidget)]
pub struct CategorySection;

impl StatelessWidget for CategorySection {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let store = ProviderHandle::<AppStore>::of(ctx);
        let theme = ForumTheme::of(ctx);

        AsyncBuilder::new()
            .key("categories")
            .future(move || {
                let store = store.clone();
                async move {
                    let root = AppStore::fetch_categories().await?;
                    info!(
                        "Categories fetched : {:?}",
                        root.category_list.categories.len()
                    );
                    store.update(|s| {
                        s.cache_categories(&root.category_list.categories)
                    });
                    Ok::<_, String>(root)
                }
            })
            .child(move |snapshot| match snapshot {
                AsyncSnapshot::Waiting => category_list(
                    &theme,
                    Column::new()
                        .gaps(LayoutSpacing::all(2))
                        .list(0..9)
                        .builder(|_| SkeletonCategoryRow::new())
                        .boxed(),
                ),
                AsyncSnapshot::Error(error) => category_list(
                    &theme,
                    Text::new(error.clone())
                        .text_style(TextStyle::new().color(theme.on_surface_color()))
                        .boxed(),
                ),
                AsyncSnapshot::Data(data) => {
                    let categories = data
                        .category_list
                        .categories
                        .iter()
                        .map(|category| category.as_share_ref())
                        .collect::<Vec<_>>();

                    category_list(
                        &theme,
                        Column::new()
                            .gaps(LayoutSpacing::all(2))
                            .list(categories)
                            .builder(CategoryRow::new)
                            .boxed()
                    )
                }
            })
    }
}

fn category_list(theme: &ForumTheme, body: AnyWidget) -> AnyWidget {
    CollapsibleList::new()
        .animation_millis(300)
        .header(
            ListHeader::new().child(
                Text::new("Categories").text_style(
                    TextStyle::new()
                        .font_weight(FontWeight::Bold)
                        .font_size(20)
                        .color(theme.on_surface_color()),
                ),
            ),
        )
        .body(ListBody::new().child(body))
        .boxed()
}
