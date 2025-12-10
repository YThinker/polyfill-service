use indexmap::IndexMap;
use maud::{html, Markup, Render, DOCTYPE};
use serde::Deserialize;
use std::{collections::HashMap, fmt::Write as _};

struct Raw(String);

impl Render for Raw {
    fn render_to(&self, output: &mut String) {
        write!(output, "{}", self.0).unwrap();
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolyfillData {
    polyfills: Vec<String>,
    polyfill_aliases: HashMap<String, Vec<String>>,
    // version: String,
}

fn layout(content: Markup) -> Markup {
    let stylee = Raw(include_str!("style.css").to_string());
    let scriptt = Raw(include_str!("script.js").to_string());
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta http-equiv="x-ua-compatible" content="ie=edge";
                meta name="description" content="cdnjs.cloudflare.com/polyfill is a service which accepts a request for a set of browser features and returns only the polyfills that are needed by the requesting browser.";
                meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no";
                title {"cdnjs.cloudflare.com/polyfill";}
                link rel="icon" type="image/svg" href="/img/logo.svg";
                style {(stylee);};
            }
            body {
                (content)
                script defer="" {
                    (scriptt);
                }
            }
        }
    }
}

pub(crate) fn home() -> String {
    let data: PolyfillData = serde_json::from_str(include_str!("json/library-4.8.0.json")).unwrap();
    let mut aliases: IndexMap<String, Vec<String>> = data.polyfill_aliases.into_iter().collect();
    aliases.sort_keys();
    layout(html! {
        header {
            div class="container" {
                h1 {"Upgrade the web. Automatically.";}
                p {
                    "Delivers only the polyfills required by the user's web browser."
                }
            }
        }
        main class="container" {
            form {
                label for="bundle" {
                    "Your polyfill bundle";
                    output {
                        pre {
                            code id="polyfill-bundle-url" {
                                ""
                            }
                        }
                    }
                }
                div class="grid" {

                    label for="library-version" {
                        "Polyfill Library Version";
                        select id="library-version" {
                            option value="4.8.0" selected="" {"4.8.0";}
                            option value="3.111.0" {"3.111.0";}
                            option value="3.101.0" {"3.101.0";}
                            option value="3.103.0" {"3.103.0";}
                            option value="3.104.0" {"3.104.0";}
                            option value="3.108.0" {"3.108.0";}
                            option value="3.109.0" {"3.109.0";}
                            option value="3.110.1" {"3.110.1";}
                            option value="3.27.4"  {"3.27.4";}
                            option value="3.34.0"  {"3.34.0";}
                            option value="3.39.0"  {"3.39.0";}
                            option value="3.40.0"  {"3.40.0";}
                            option value="3.41.0"  {"3.41.0";}
                            option value="3.42.0"  {"3.42.0";}
                            option value="3.46.0"  {"3.46.0";}
                            option value="3.48.0"  {"3.48.0";}
                            option value="3.50.2"  {"3.50.2";}
                            option value="3.51.0"  {"3.51.0";}
                            option value="3.52.0"  {"3.52.0";}
                            option value="3.52.1"  {"3.52.1";}
                            option value="3.52.2"  {"3.52.2";}
                            option value="3.52.3"  {"3.52.3";}
                            option value="3.53.1"  {"3.53.1";}
                            option value="3.89.4"  {"3.89.4";}
                            option value="3.96.0"  {"3.96.0";}
                            option value="3.98.0"  {"3.98.0";}
                        }
                    }

                    label for="callback" {
                        "Callback";
                        input type="text" id="callback" name="callback";
                    }

                }

                label for="filter-polyfills" {"Filter Polyfills";}
                input type="text" id="filter-polyfills" name="filter-polyfills";
                small{"Filter the polyfills in the \"Available Polyfills\" list."}

                fieldset {
                    legend {"Available Polyfills";}
					small{"Check the boxes of the polyfills or polyfill-sets you want to have in your bundle.";}

                    div id="features-list" {
                        @for alias in &aliases {
                            div class="feature" {
                                label for=(alias.0) {
                                    input type="checkbox" id=(alias.0) name=(alias.0);
                                    (alias.0);
                                }
                                details class="alias" {
                                    summary{"Included Polyfills";}
                                    ul{
                                        @for polyfill in alias.1 {
                                            li{(polyfill);}
                                        }
                                    }
                                }
                            }
                        }
                        @for polyfill in &data.polyfills {
                            div class="feature" {
                                label for=(polyfill) {
                                    input type="checkbox" id=(polyfill) name=(polyfill);
                                    (polyfill);
                                }
                            }
                        }

                    }
                }
            }
            button class="contrast switcher theme-switcher" aria-label="Turn off dark mode" {
                i{"Turn off dark mode"}
            }
        }
        script {
            (Raw(include_str!("builder.js").to_string()));
        }
    }).into_string()
}
