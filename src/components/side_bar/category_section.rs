use crate::components::side_bar::category_row::CategoryRow;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::console::info;
use aimer::style::{LayoutSpacing, TextStyle, Theme};
use aimer::{AsyncBuilder, AsyncSnapshot, BuildContext, Column, FontWeight, ProviderHandle, SizedBox, StatelessWidget, Text, Widget};
use crate::components::side_bar::skeleton_category_row::SkeletonCategoryRow;

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
                    store.update(|s| s.cache_categories(root.category_list.categories.clone()));
                    Ok::<_, String>(root)
                }
            })
            .child(move |snapshot| match snapshot {
                AsyncSnapshot::Waiting => Column::new()
                    .children([
                        Text::new("Categories")
                            .text_style(
                                TextStyle::new()
                                    .font_weight(FontWeight::Bold)
                                    .font_size(20)
                                    .color(theme.on_surface_color()),
                            )
                            .boxed(),
                        SizedBox::new().height(12).boxed(),
                        Column::new()
                            .gaps(LayoutSpacing::all(2))
                            .list(0..9)
                            .builder(|_| SkeletonCategoryRow::new())
                            .boxed(),
                    ])
                    .boxed(),
                AsyncSnapshot::Error(error) => Text::new(error.clone())
                    .text_style(TextStyle::new().color(theme.on_surface_color()))
                    .boxed(),
                AsyncSnapshot::Data(data) => {
                    let categories = data
                        .category_list
                        .categories
                        .iter()
                        // .filter(|category| !category.is_uncategorized)
                        .cloned()
                        .collect::<Vec<_>>();

                    Column::new()
                        .children([
                            Text::new("Categories")
                                .text_style(
                                    TextStyle::new()
                                        .font_weight(FontWeight::Bold)
                                        .font_size(20)
                                        .color(theme.on_surface_color()),
                                )
                                .boxed(),
                            SizedBox::new().height(12).boxed(),
                            Column::new()
                                .gaps(LayoutSpacing::all(2))
                                .list(categories)
                                .builder(CategoryRow::new)
                                .boxed(),
                        ])
                        .boxed()
                }
            })
    }
}

