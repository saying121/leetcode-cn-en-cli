use std::path::PathBuf;

use miette::{IntoDiagnostic, Result};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{instrument, trace};

use crate::{
    config::global::glob_config,
    dao::get_question_index_exact,
    entities::*,
    leetcode::{qs_detail::Question, IdSlug},
    render::Render,
};

/// write info to file
pub struct CacheFile {
    pub code_path:      PathBuf,
    pub test_case_path: PathBuf,
    pub content_path:   PathBuf,
}

impl CacheFile {
    /// Get code, test, content dir
    #[instrument]
    pub async fn new(idslug: &IdSlug) -> Result<Self> {
        let pb: index::Model = get_question_index_exact(idslug).await?;
        let user_config = glob_config();
        let mut cache_path = user_config.config.code_dir.clone();
        let sub_dir = format!("{}_{}", pb.question_id, pb.question_title_slug,);
        cache_path.push(sub_dir);
        create_dir_all(&cache_path)
            .await
            .into_diagnostic()?;

        let mut code_path = cache_path.clone();
        let code_file_name = format!("{}{}", pb.question_id, user_config.get_suffix());
        code_path.push(code_file_name);
        trace!("code path: {:?}", code_path);

        let mut test_case_path = cache_path.clone();
        let test_file_name = format!("{}_test_case.txt", pb.question_id);
        test_case_path.push(test_file_name);
        trace!("test case path: {:?}", test_case_path);

        let mut content_path = cache_path.clone();
        let temp = if glob_config().config.translate {
            "cn"
        }
        else {
            "en"
        };
        let detail_file_name = format!("{}_detail_{}.md", pb.question_id, temp);
        content_path.push(detail_file_name);
        trace!("content case path: {:?}", content_path);
        Ok(Self {
            code_path,
            test_case_path,
            content_path,
        })
    }
    /// Write a question's code and test case to file
    pub async fn write_to_file(&self, detail: Question) -> Result<()> {
        let user = glob_config();

        let content = detail.to_md_str(true);
        let (r1, r2) = tokio::join!(
            Self::write_file(&self.test_case_path, &detail.example_testcases),
            Self::write_file(&self.content_path, &content)
        );
        r1?;
        r2?;

        for code_snippet in &detail
            .code_snippets
            .as_ref()
            .cloned()
            .unwrap_or_default()
        {
            if code_snippet.lang_slug == user.config.lang {
                #[rustfmt::skip]
                let (start,end,mut inject_start,inject_end) = user.get_lang_info();
                if !inject_start.is_empty() {
                    inject_start += "\n";
                }
                let code_str = format!(
                    "{}{}\n{}\n{}\n{}",
                    inject_start, start, code_snippet.code, end, inject_end
                );
                Self::write_file(&self.code_path, &code_str).await?;
            }
        }

        // if this question not support this lang, or is paid only
        if !self.code_path.exists() {
            let mut temp;

            if detail.is_paid_only {
                temp = "this question is paid only".to_owned();
            }
            else {
                temp = "this question not support the lang or \n\nsupport below:\n".to_owned();
                for code_snippet in &detail
                    .code_snippets
                    .as_ref()
                    .cloned()
                    .unwrap_or_default()
                {
                    temp += &format!("{}\n", code_snippet.lang_slug);
                }
            }

            Self::write_file(&self.code_path, &temp).await?;
        }

        Ok(())
    }
    pub async fn get_user_code(&self, idslug: &IdSlug) -> Result<(String, String)> {
        let (code_file, test_case_file) = tokio::join!(
            File::open(&self.code_path),
            File::open(&self.test_case_path)
        );
        let (mut code_file, mut test_case_file) = (
            code_file.map_err(|err| {
                miette::miette!(
                    "Error: {}. There is no code file, maybe you changed the name, please get \
                     **{}** question detail again",
                    err,
                    idslug
                )
            })?,
            test_case_file.map_err(|err| {
                miette::miette!(
                    "Error: {}. There is no test case file, maybe you changed the name, please \
                     remove relate file and get **{}** question detail again, or manual create a \
                     same name blank file",
                    err,
                    idslug
                )
            })?,
        );

        let mut code = String::new();
        let mut test_case = String::new();

        let (code_res, test_case_res) = tokio::join!(
            code_file.read_to_string(&mut code),
            test_case_file.read_to_string(&mut test_case)
        );
        _ = (
            code_res.into_diagnostic()?,
            test_case_res.into_diagnostic()?,
        );
        Ok((code, test_case))
    }

    /// create file and write something
    async fn write_file(path: &PathBuf, val: &str) -> Result<()> {
        if !path.exists() {
            create_dir_all(&path.parent().unwrap())
                .await
                .into_diagnostic()
                .unwrap();

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&path)
                .await
                .into_diagnostic()?;
            file.write_all(val.as_bytes())
                .await
                .into_diagnostic()?;

            file.sync_all()
                .await
                .into_diagnostic()?;
        }
        Ok(())
    }
}
