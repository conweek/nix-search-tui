use std::process::Command;
use serde_json::Value;

pub enum CurrScreen {
    Searching,
    DisplayResults,
    Both,
    Detail,
}

#[derive(PartialEq)]
pub enum Search {
    Package,
    Configuration,
    HomeConfiguration,
}

pub struct App {
    pub search_option: String,
    pub results: Vec<String>,
    pub current_screen: CurrScreen,
    pub search_choice: Search,
    pub selected_result: usize,
    pub detail: String,
    pub results_from_config: bool,
    man_cache: Option<String>,
}

// App Methods
impl App {
    pub fn new() -> App {
        App {
            search_option: String::new(),
            results: Vec::new(),
            current_screen: CurrScreen::Searching,
            search_choice: Search::Package,
            selected_result: 0,
            detail: String::new(),
            results_from_config: false,
            man_cache: None,
        }
    }

    fn get_man_page(&mut self, file: &str) -> Result<&str, Box<dyn std::error::Error>> {
        if self.man_cache.is_none() {
            let output = Command::new("man")
                .args(["-P", "cat", file])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .output()?;
            self.man_cache = Some(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        Ok(self.man_cache.as_deref().unwrap())
    }

    pub fn search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.results_from_config = matches!(self.search_choice, Search::Configuration);
        match self.search_choice {
            Search::Package => {
                let body = serde_json::json!({
                    "from": 0,
                    "size": 1000,
                    "sort": [
                        {"_score": "desc", "package_attr_name": "desc", "package_pversion": "desc"}
                    ],
                    "query": {
                        "bool": {
                            "must": [
                                {"match": {"type": "package"}},
                                {"multi_match": {
                                    "type": "cross_fields",
                                    "query": self.search_option,
                                    "analyzer": "whitespace",
                                    "auto_generate_synonyms_phrase_query": false,
                                    "operator": "and",
                                    "fields": [
                                        "package_attr_name^9",
                                        "package_pname^6",
                                        "package_description^1.3",
                                        "package_programs^9"
                                    ]
                                }}
                            ]
                        }
                    }
                });

                // Adapted from https://github.com/peterldowns/nix-search-cli
                // full credit to peterldowns for this!!
                let output = Command::new("curl")
                    .arg("-s")
                    .arg("-u")
                    .arg("aWVSALXpZv:X8gPHnzL52wFEekuxsfQ9cSh")
                    .arg("https://nixos-search-7-1733963800.us-east-1.bonsaisearch.net:443/latest-*-nixos-unstable/_search")
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-d")
                    .arg(body.to_string())
                    .output()?;

                let resp: Value = serde_json::from_slice(&output.stdout)?;

                self.results.clear();
                if let Some(hits) = resp["hits"]["hits"].as_array() {
                    for hit in hits {
                        let name = hit["_source"]["package_attr_name"].as_str().unwrap_or("?");
                        let desc = hit["_source"]["package_description"].as_str().unwrap_or("");
                        self.results.push(format!("{name} - {desc}"));
                    }
                }

            }
            Search::Configuration => {
                let text = self.get_man_page("configuration.nix")?.to_owned();
                let query = &self.search_option;

                self.results.clear();
                for line in text.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty()
                        && !trimmed.contains(' ')
                        && trimmed.contains(query.as_str())
                        && (trimmed.starts_with("programs.") || trimmed.starts_with("services."))
                    {
                        self.results.push(trimmed.to_string());
                    }
                }
            }
            Search::HomeConfiguration => {
                let text = self.get_man_page("home-configuration.nix")?.to_owned();
                let query = &self.search_option;

                self.results.clear();
                if text.is_empty() {
                    self.results.push("Home manager not found".to_string());
                } else {
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty()
                            && !trimmed.contains(' ')
                            && trimmed.contains(query.as_str())
                            && (trimmed.starts_with("programs.") || trimmed.starts_with("services."))
                        {
                            self.results.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn fetch_detail(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(selected) = self.results.get(self.selected_result) else {
            return Ok(());
        };
        let option_name = selected.trim().to_string();

        let man_page = match self.search_choice {
            Search::Configuration => "configuration.nix",
            Search::HomeConfiguration => "home-configuration.nix",
            _ => return Ok(()),
        };
        let text = self.get_man_page(man_page)?.to_owned();

        let mut found = false;
        let mut entry = String::new();

        for line in text.lines() {
            if line.trim() == option_name {
                found = true;
                entry.push_str(line);
                entry.push('\n');
            } else if found {
                if line.trim().is_empty() {
                    break;
                }
                entry.push_str(line);
                entry.push('\n');
            }
        }

        self.detail = if entry.is_empty() {
            format!("No man page entry found for '{option_name}'")
        } else {
            entry
        };

        Ok(())
    }

    pub fn print_results(&self) {
        for element in &self.results {
            println!("Found: {element}");
        }
    }

    pub fn cycle_tab(&mut self) {
        self.man_cache = None;
        self.search_choice = match self.search_choice {
            Search::Package => Search::Configuration,
            Search::Configuration => Search::HomeConfiguration,
            Search::HomeConfiguration => Search::Package,
        };
    }
}
