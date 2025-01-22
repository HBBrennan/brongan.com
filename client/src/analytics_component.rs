use globe::{Canvas, GlobeConfig, GlobeTemplate};
use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use shared::Analytics;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement};
use yew::html;
use yew::html::HtmlResult;
use yew::suspense::use_future;
use yew::Suspense;
use yew::{
    function_component, use_effect_with_deps, use_node_ref, use_state, Callback, Html, Properties,
};

fn format_count(count: &i32) -> Html {
    html! {
        <span class="count-badge">{ count }</span>
    }
}

fn to_html(analytics: &Analytics) -> Html {
    html! {
        <tr>
            <td class="ip-cell">{ &analytics.ip_address }</td>
            <td class="path-cell">
                <span class="path-prefix">{"/"}</span>
                { analytics.path.trim_start_matches('/') }
            </td>
            <td class="country-cell">
                <span class="country-code">{ &analytics.iso_code }</span>
            </td>
            <td class="count-cell">{ format_count(&(analytics.count as i32)) }</td>
        </tr>
    }
}

#[function_component(LoadingSpinner)]
fn loading_spinner() -> Html {
    html! {
        <div class="loading-container">
            <div class="loading-spinner"></div>
            <span class="loading-text">{"Loading analytics..."}</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    on_filter_change: Callback<String>,
    on_search_change: Callback<String>,
    on_view_change: Callback<ViewType>,
    on_export: Callback<()>,
    view_type: ViewType,
    data: Vec<Analytics>,
}

#[derive(PartialEq, Clone)]
pub enum ViewType {
    Table,
    Globe,
}

#[derive(PartialEq, Clone)]
pub enum SortField {
    IpAddress,
    Path,
    Country,
    Visits,
}

#[derive(PartialEq, Clone)]
pub enum SortDirection {
    Ascending,
    Descending,
}

fn sort_analytics(data: &mut [Analytics], field: &SortField, direction: &SortDirection) {
    data.sort_by(|a, b| {
        let ordering = match field {
            SortField::IpAddress => a.ip_address.cmp(&b.ip_address),
            SortField::Path => a.path.cmp(&b.path),
            SortField::Country => a.iso_code.cmp(&b.iso_code),
            SortField::Visits => a.count.cmp(&b.count),
        };
        match direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    });
}

#[function_component(AnalyticsToolbar)]
fn analytics_toolbar(props: &ToolbarProps) -> Html {
    html! {
        <div class="tools-section">
            <div class="search-filter-group">
                <div class="search-group">
                    <label>{"Search:"}</label>
                    <input
                        type="text"
                        placeholder="Filter by IP or path..."
                        onchange={props.on_search_change.reform(|e: Event| {
                            let input = e.target().unwrap().unchecked_into::<HtmlInputElement>();
                            input.value()
                        })}
                    />
                </div>
                <div class="filter-group">
                    <label>{"Country:"}</label>
                    <select onchange={props.on_filter_change.reform(|e: Event| {
                        let select = e.target().unwrap().unchecked_into::<HtmlSelectElement>();
                        select.value()
                    })}>
                        <option value="">{"All Countries"}</option>
                        <option value="US">{"United States"}</option>
                        <option value="GB">{"United Kingdom"}</option>
                        // Add more countries as needed
                    </select>
                </div>
            </div>
            <div class="view-controls">
                <div class="view-toggle">
                    <button
                        class={if props.view_type == ViewType::Table { "active" } else { "" }}
                        onclick={props.on_view_change.reform(|_| ViewType::Table)}>
                        {"Table View"}
                    </button>
                    <button
                        class={if props.view_type == ViewType::Globe { "active" } else { "" }}
                        onclick={props.on_view_change.reform(|_| ViewType::Globe)}>
                        {"Globe View"}
                    </button>
                </div>
                <button
                    class="export-button"
                    onclick={props.on_export.reform(|_| ())}>
                    {"Export CSV"}
                </button>
            </div>
        </div>
    }
}

#[function_component(AnalyticsHeader)]
fn analytics_header() -> Html {
    html! {
        <div class="header">
            <div class="title-section">
                <h1>{"Analytics Dashboard"}</h1>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct AsciiGlobeProps {
    data: Vec<Analytics>,
}

#[function_component(AsciiGlobe)]
fn ascii_globe(props: &AsciiGlobeProps) -> Html {
    let rotation = use_state(|| 0.0f64);
    let globe_ref = use_node_ref();

    // Set up globe rotation animation
    {
        let rotation = rotation.clone();
        use_effect_with_deps(
            move |_| {
                let interval = Interval::new(50, move || {
                    rotation.set(*rotation + 0.02);
                });
                || {
                    interval.forget();
                }
            },
            (),
        );
    }

    // Update globe rendering
    {
        let globe_ref = globe_ref.clone();
        let rotation_val = *rotation;

        use_effect_with_deps(
            move |_| {
                if let Some(pre) = globe_ref.cast::<web_sys::Element>() {
                    // Create and configure globe with correct dimensions
                    let globe = GlobeConfig::new()
                        .use_template(GlobeTemplate::Earth)
                        .build();

                    // Create canvas with correct dimensions (250x250 as per docs)
                    let mut canvas = Canvas::new(250, 250, None);
                    globe.render_on(&mut canvas);

                    // Convert to HTML string using correct matrix access
                    let mut globe_str = String::new();
                    let (size_x, size_y) = canvas.get_size();

                    // Use correct character size (4x8) for iteration
                    for i in 0..size_y / 8 {
                        for j in 0..size_x / 4 {
                            globe_str.push_str(&canvas.matrix[i][j].to_string());
                        }
                        globe_str.push('\n');
                    }

                    // Update the pre element
                    pre.set_inner_html(&globe_str);
                }
                || ()
            },
            rotation_val,
        );
    }

    // Visitor dots overlay with adjusted positioning for 250x250 canvas
    let country_positions = [
        ("US", (40.0, 35.0)), // United States
        ("GB", (48.0, 30.0)), // United Kingdom
        ("DE", (50.0, 30.0)), // Germany
        ("FR", (48.0, 32.0)), // France
        ("ES", (46.0, 33.0)), // Spain
        ("IT", (50.0, 33.0)), // Italy
        ("JP", (70.0, 35.0)), // Japan
        ("CN", (65.0, 35.0)), // China
        ("IN", (60.0, 40.0)), // India
        ("AU", (68.0, 55.0)), // Australia
        ("BR", (42.0, 50.0)), // Brazil
        ("CA", (40.0, 28.0)), // Canada
        ("RU", (58.0, 28.0)), // Russia
        ("ZA", (52.0, 55.0)), // South Africa
        ("MX", (35.0, 40.0)), // Mexico
        ("AR", (40.0, 58.0)), // Argentina
    ];

    let visitor_dots = props
        .data
        .iter()
        .map(|analytics| {
            if let Some(&(_, (mut x, y))) = country_positions
                .iter()
                .find(|(code, _)| *code == analytics.iso_code.as_str())
            {
                // Adjust x position based on rotation
                x = (x + *rotation * 20.0) % 100.0;
                if x < 20.0 || x > 80.0 {
                    return html! {}; // Don't show dots on the "back" of the globe
                }

                let background = if analytics.count > 10 {
                    "var(--chart-accent)"
                } else {
                    "#7b5fff"
                };

                let style = format!(
                    "left: {}%; top: {}%; background: {}; opacity: {};",
                    x,
                    y,
                    background,
                    // Fade dots near the edges
                    if x < 30.0 || x > 70.0 { 0.5 } else { 1.0 }
                );

                html! {
                    <div class="visitor-dot" {style}>
                        <div class="visitor-pulse"></div>
                    </div>
                }
            } else {
                html! {}
            }
        })
        .collect::<Html>();

    html! {
        <div class="ascii-globe">
            <div class="globe-container">
                <pre ref={globe_ref} class="globe-text"></pre>
                { visitor_dots }
            </div>
            <div class="globe-legend">
                <div class="legend-item active">
                    <span class="dot"></span>
                    <span>{"Active Visitors"}</span>
                </div>
                <div class="legend-item recent">
                    <span class="dot"></span>
                    <span>{"Recent Visitors"}</span>
                </div>
            </div>
        </div>
    }
}

#[function_component(AnalyticsContent)]
fn analytics_content() -> HtmlResult {
    let view_type = use_state(|| ViewType::Table);
    let country_filter = use_state(String::new);
    let search_filter = use_state(String::new);
    let sort_field = use_state(|| SortField::Visits);
    let sort_direction = use_state(|| SortDirection::Descending);
    let current_page = use_state(|| 1);
    let items_per_page = 10;

    let resp = use_future(|| async {
        Request::get("/api/analytics")
            .send()
            .await?
            .json::<Vec<Analytics>>()
            .await
    })?;

    match *resp {
        Ok(ref res) => {
            let mut filtered_data = res
                .iter()
                .filter(|a| {
                    let country_match =
                        country_filter.is_empty() || &a.iso_code == &*country_filter;
                    let search_match = search_filter.is_empty()
                        || a.ip_address
                            .to_lowercase()
                            .contains(&search_filter.to_lowercase())
                        || a.path
                            .to_lowercase()
                            .contains(&search_filter.to_lowercase());
                    country_match && search_match
                })
                .cloned()
                .collect::<Vec<Analytics>>();

            // Sort the data
            sort_analytics(&mut filtered_data, &sort_field, &sort_direction);

            // Calculate pagination
            let total_pages = (filtered_data.len() as f32 / items_per_page as f32).ceil() as usize;
            let start_idx = ((*current_page - 1) * items_per_page) as usize;
            let end_idx = start_idx + items_per_page;
            let paginated_data = filtered_data
                [start_idx.min(filtered_data.len())..end_idx.min(filtered_data.len())]
                .to_vec();

            let analytics_table = paginated_data.iter().map(|a| to_html(a)).collect::<Html>();

            let sort_header = |field: SortField, label: &str| {
                let is_active = &field == &*sort_field;
                let direction = if is_active {
                    match *sort_direction {
                        SortDirection::Ascending => "↑",
                        SortDirection::Descending => "↓",
                    }
                } else {
                    ""
                };

                let onclick = {
                    let sort_field = sort_field.clone();
                    let sort_direction = sort_direction.clone();
                    let field = field.clone();
                    Callback::from(move |_| {
                        if &field == &*sort_field {
                            sort_direction.set(match *sort_direction {
                                SortDirection::Ascending => SortDirection::Descending,
                                SortDirection::Descending => SortDirection::Ascending,
                            });
                        } else {
                            sort_field.set(field.clone());
                            sort_direction.set(SortDirection::Ascending);
                        }
                    })
                };

                html! {
                    <th class={if is_active { "active" } else { "" }} {onclick}>
                        <span class="sort-header">
                            { label }
                            <span class="sort-indicator">{ direction }</span>
                        </span>
                    </th>
                }
            };

            Ok(html! {
                <div class="analytics">
                    <AnalyticsHeader />
                    <AnalyticsToolbar
                        on_filter_change={Callback::from(move |c| country_filter.set(c))}
                        on_search_change={
                            let search_filter = search_filter.clone();
                            Callback::from(move |s| search_filter.set(s))
                        }
                        on_view_change={
                            let view_type = view_type.clone();
                            Callback::from(move |v| view_type.set(v))
                        }
                        on_export={
                            let filtered_data = filtered_data.clone();
                            Callback::from(move |_| {
                                let csv = export_to_csv(&filtered_data);
                                download_csv(&csv);
                            })
                        }
                        view_type={(*view_type).clone()}
                        data={filtered_data.clone()}
                    />
                    {
                        if *view_type == ViewType::Table {
                            html! {
                                <>
                                    <div class="table-container">
                                        <table>
                                            <thead>
                                                <tr>
                                                    { sort_header(SortField::IpAddress, "IP Address") }
                                                    { sort_header(SortField::Path, "Path") }
                                                    { sort_header(SortField::Country, "Country") }
                                                    { sort_header(SortField::Visits, "Visits") }
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { analytics_table }
                                            </tbody>
                                        </table>
                                    </div>
                                    <div class="pagination">
                                        <button
                                            class="pagination-button"
                                            disabled={*current_page == 1}
                                            onclick={
                                                let current_page = current_page.clone();
                                                Callback::from(move |_| {
                                                    if *current_page > 1 {
                                                        current_page.set(*current_page - 1);
                                                    }
                                                })
                                            }
                                        >
                                            {"Previous"}
                                        </button>
                                        <span class="page-info">
                                            {format!("Page {} of {}", *current_page, total_pages)}
                                        </span>
                                        <button
                                            class="pagination-button"
                                            disabled={*current_page >= total_pages}
                                            onclick={
                                                let current_page = current_page.clone();
                                                Callback::from(move |_| {
                                                    current_page.set(*current_page + 1);
                                                })
                                            }
                                        >
                                            {"Next"}
                                        </button>
                                    </div>
                                </>
                            }
                        } else {
                            html! { <AsciiGlobe data={filtered_data} /> }
                        }
                    }
                </div>
            })
        }
        Err(ref failure) => Ok(html! {
            <div class="error-container">
                <div class="error-message">
                    <span class="error-icon">{"⚠️"}</span>
                    <p>{"Failed to load analytics data"}</p>
                    <p class="error-details">{ failure.to_string() }</p>
                </div>
            </div>
        }),
    }
}

// Function to convert analytics data to CSV
fn export_to_csv(data: &[Analytics]) -> String {
    let mut csv = String::from("IP Address,Path,Country,Visits\n");
    for analytics in data {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            analytics.ip_address, analytics.path, analytics.iso_code, analytics.count
        ));
    }
    csv
}

// Function to trigger CSV download
fn download_csv(csv_content: &str) {
    let element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("a")
        .unwrap();

    let blob =
        web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&csv_content.into())).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    element.set_attribute("href", &url).unwrap();
    element.set_attribute("download", "analytics.csv").unwrap();

    element.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
    web_sys::Url::revoke_object_url(&url).unwrap();
}

#[function_component(AnalyticsComponent)]
pub fn analytics() -> Html {
    html! {
        <Suspense fallback={html! { <LoadingSpinner /> }}>
            <AnalyticsContent />
        </Suspense>
    }
}
