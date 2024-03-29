use std::fmt::Display;

use lcode_config::config::global::G_USER_CONFIG;
#[cfg(feature = "ratatui")]
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
};

use super::{to_sub_sup_script, Render};
use crate::leetcode::question::qs_detail::Question;

impl Render for Question {
    fn to_md_str(&self, with_env: bool) -> String {
        let content = if G_USER_CONFIG.config.translate {
            self.translated_content
                .as_deref()
                .unwrap_or_default()
        }
        else {
            self.content
                .as_deref()
                .unwrap_or_default()
        };

        let content = to_sub_sup_script(content)
            .trim_matches('"')
            .replace("\\n", "\n");
        let env_info = self.env_info.to_string();

        // some content are not HTML
        let md_str = if content.contains("<p>") {
            html2text::from_read(content.as_bytes(), 80)
        }
        else {
            content
        };
        let mut res = format!("{qs}\n---\n\n{md}\n---\n", qs = self, md = md_str);

        if !self.hints.is_empty() {
            let hints = html2text::from_read(self.hints.join("\n").as_bytes(), 80);
            res = format!("{}\n\nhints:\n{}\n---\n", res, hints);
        }
        if !self.mysql_schemas.is_empty() {
            let str = format!("\n```sql\n{}\n```\n", self.mysql_schemas.join("\n"));
            res.push_str(&str);
        }
        if with_env {
            res.push_str(&format!("EnvInfo:\n{}", env_info));
        }
        res
    }

    #[cfg(feature = "ratatui")]
    fn to_tui_vec(&self) -> Vec<Line> {
        use scraper::Html;

        let content = if G_USER_CONFIG.config.translate {
            self.translated_content
                .as_deref()
                .unwrap_or_else(|| {
                    self.content
                        .as_deref()
                        .unwrap_or_default()
                })
        }
        else {
            self.content
                .as_deref()
                .unwrap_or_default()
        };

        let content = to_sub_sup_script(content);

        let frag = Html::parse_fragment(&content);
        let res = frag
            .root_element()
            .text()
            .fold(String::new(), |acc, e| acc + e);

        let res = res
            .replace("\\\"", "\"")
            .replace("\\\\", "")
            .replace("\\n", "\n")
            .replace("\\t", "    ")
            .replace("\n\n\n", "\n");

        let res = res
            .trim_matches(|c| c == '"' || c == '\n' || c == ' ')
            .split('\n')
            .map(|v| -> Line<'_> { v.to_owned().into() });

        let topic = self
            .topic_tags
            .iter()
            .map(|v| {
                if G_USER_CONFIG.config.translate {
                    if v.translated_name.is_none() {
                        v.name.clone()
                    }
                    else {
                        v.translated_name
                            .clone()
                            .unwrap_or_default()
                    }
                }
                else {
                    v.name.clone()
                }
            })
            .fold(String::new(), |acc, v| format!("{}, {}", acc, v));

        let mut res1 = vec![
            vec![
                Span::styled("• ID: ", Style::default()),
                Span::styled(self.question_id.clone(), Style::default().bold()),
                Span::styled(" | Passing rate: ", Style::default()),
                Span::styled(self.stats.ac_rate.clone(), Style::default().bold()),
                Span::styled(" | PaidOnly: ", Style::default()),
                Span::styled(self.is_paid_only.to_string(), Style::default().bold()),
                Span::styled(" | Difficulty: ", Style::default()),
                Span::styled(self.difficulty.clone(), Style::default().bold()),
            ]
            .into(),
            vec![
                Span::styled("• Topic: ", Style::default().bold()),
                Span::styled(topic, Style::default()),
            ]
            .into(),
            vec![
                Span::styled("• Url: ", Style::default()),
                Span::styled(
                    G_USER_CONFIG.urls.get_qs_url(
                        self.qs_slug
                            .as_deref()
                            .unwrap_or_default(),
                    ),
                    Style::default().bold(),
                ),
            ]
            .into(),
            String::new().into(),
        ];
        res1.extend(res);

        res1
    }
}

impl Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = if G_USER_CONFIG.config.translate {
            self.translated_title
                .as_ref()
                .map_or(self.title.clone(), std::clone::Clone::clone)
                .as_str()
                .trim_matches('"')
                .to_owned()
        }
        else {
            self.title.clone()
        };

        let topic = self
            .topic_tags
            .iter()
            .map(|v| {
                if G_USER_CONFIG.config.translate {
                    if v.translated_name.is_none() {
                        v.name.clone()
                    }
                    else {
                        v.translated_name
                            .clone()
                            .unwrap_or_default()
                    }
                }
                else {
                    v.name.clone()
                }
            })
            .fold(String::new(), |acc, v| format!("{}, {}", acc, v));

        let t_case = format!("```\n{}\n```", self.example_testcases);
        format!(
            "# {tit:62}\n\n* ID: {id:07} | Passing rate: {rt:.6} | PaidOnly: {pd:6} | Difficulty: \
             {di}\n* Url: {url}\n* Topic: {tp}\n\n## Test Case:\n\n{t_case}\n",
            tit = title,
            id = self.question_id,
            rt = self.stats.ac_rate,
            pd = self.is_paid_only,
            di = self.difficulty,
            tp = topic,
            t_case = t_case,
            url = G_USER_CONFIG.urls.get_qs_url(
                self.qs_slug
                    .as_deref()
                    .unwrap_or_default()
            )
        )
        .fmt(f)
    }
}
