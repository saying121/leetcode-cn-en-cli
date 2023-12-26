mod tab0;
mod tab1;
mod tab2;
mod tab3;

use std::sync::LazyLock;

use crossterm::event::{Event as CrossEvent, KeyEvent};
use miette::{IntoDiagnostic, Result};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};
use tracing::error;
use tui_textarea::TextArea;

pub use super::keymap::TuiMode;
use super::{keymap::*, myevent::UserEvent};
use crate::{
    config::global::glob_leetcode,
    dao::save_info::CacheFile,
    editor::{edit, CodeTestFile},
    leetcode::{
        qs_detail::Question,
        resps::{run_res::RunResult, SubmitInfo, TestInfo},
        IdSlug,
    },
    render::Render,
};

#[derive(PartialEq, Eq)]
pub enum Tab2Panel {
    AllTopics,
    UserTopics,
    Difficulty,
    Questions,
}

impl Tab2Panel {
    fn left(&mut self) {
        *self = match self {
            Self::AllTopics => Self::AllTopics,
            Self::UserTopics => Self::UserTopics,
            Self::Difficulty => Self::AllTopics,
            Self::Questions => Self::Difficulty,
        }
    }
    fn right(&mut self) {
        *self = match self {
            Self::AllTopics => Self::Difficulty,
            Self::UserTopics => Self::Difficulty,
            Self::Difficulty => Self::Questions,
            Self::Questions => Self::Questions,
        }
    }
    fn up(&mut self) {
        *self = match self {
            Self::AllTopics => Self::AllTopics,
            Self::UserTopics => Self::AllTopics,
            Self::Difficulty => Self::Difficulty,
            Self::Questions => Self::Questions,
        }
    }
    fn down(&mut self) {
        *self = match self {
            Self::AllTopics => Self::UserTopics,
            Self::UserTopics => Self::UserTopics,
            Self::Difficulty => Self::Difficulty,
            Self::Questions => Self::Questions,
        }
    }
}
#[derive(Clone, Copy)]
pub enum TabIndex {
    Tab0,
    Tab1,
    Tab2,
    Tab3,
}

impl From<TabIndex> for usize {
    fn from(val: TabIndex) -> Self {
        match val {
            TabIndex::Tab0 => 0,
            TabIndex::Tab1 => 1,
            TabIndex::Tab2 => 2,
            TabIndex::Tab3 => 3,
        }
    }
}

impl TabIndex {
    fn next(&mut self) {
        *self = match self {
            Self::Tab0 => Self::Tab1,
            Self::Tab1 => Self::Tab2,
            Self::Tab2 => Self::Tab3,
            Self::Tab3 => Self::Tab0,
        };
    }
    fn prev(&mut self) {
        *self = match self {
            Self::Tab0 => Self::Tab3,
            Self::Tab1 => Self::Tab0,
            Self::Tab2 => Self::Tab1,
            Self::Tab3 => Self::Tab2,
        };
    }
}

#[derive(Default)]
pub struct NextKey {
    keys:  Vec<&'static KeyMap>,
    /// current tap times
    times: usize,
}

pub static KEYMAP: LazyLock<TuiKeyMap> = LazyLock::new(TuiKeyMap::default);

impl NextKey {
    pub fn store_next(&mut self, keyevent: KeyEvent) {
        self.keys = KEYMAP
            .keymap
            .iter()
            .filter(|v| v.keys.len() > 1 && v.keys[0] == keyevent.into())
            .collect();
    }
    pub fn have_next(&self) -> bool {
        !self.keys.is_empty()
    }
    pub fn handle_key(&mut self, keyevent: KeyEvent) {
        self.times += 1;
    }
}

pub struct App<'app> {
    pub titles:    Vec<&'app str>,
    pub tab_index: TabIndex,

    pub tab0: tab0::SelectQS<'app>,
    pub tab1: tab1::EditCode<'app>,
    pub tab2: tab2::TopicTagsQS<'app>,
    pub tab3: tab3::KeyMaps<'app>,

    pub cur_qs:          Question,

    pub tx: mpsc::UnboundedSender<UserEvent>,

    pub pop_temp: bool,
    pub temp_str: String,

    pub save_code: bool,

    pub next_key: NextKey,
}
impl<'app_lf> App<'app_lf> {
    /// do a action
    pub async fn handle_key(&mut self, keyevent: KeyEvent) {
        if self.next_key.have_next() {
            self.next_key.handle_key(keyevent);
        }

        let res = match self.tab_index {
            TabIndex::Tab0 => {
                self.handle_key_tab0(keyevent)
                    .await
            },
            TabIndex::Tab1 => {
                self.handle_key_tab1(keyevent)
                    .await
            },
            TabIndex::Tab2 => {
                self.handle_key_tab2(keyevent)
                    .await
            },
            TabIndex::Tab3 => {
                self.handle_key_tab3(keyevent)
                    .await
            },
        };
        if res {
            self.render();
        }
    }
    async fn handle_key_tab3(&mut self, keyevent: KeyEvent) -> bool {
        for KeyMap {
            keys,
            r#do,
            ..
        } in &KEYMAP.keymap
        {
            if keys.is_empty() || keys[0] != keyevent.into() {
                continue;
            }
            if keys.len() > 1 {
                self.next_key.store_next(keyevent);
            }
            else {
                self.do_action(r#do).await.unwrap();
            }
        }
        true
    }
    #[allow(renamed_and_removed_lints)]
    #[allow(unused_async)]
    async fn handle_key_tab2(&mut self, keyevent: KeyEvent) -> bool {
        if matches!(self.tab2.input_line_mode, TuiMode::Insert) {
            self.tab2
                .keymap_insert(CrossEvent::Key(keyevent));
            return true;
        }

        for KeyMap {
            keys,
            r#do,
            ..
        } in &KEYMAP.keymap
        {
            if keys.is_empty() || keys[0] != keyevent.into() {
                continue;
            }
            if keys.len() > 1 {
                self.next_key.store_next(keyevent);
            }
            else {
                self.do_action(r#do).await.unwrap();
            }
        }
        true
    }
    async fn handle_key_tab1(&mut self, keyevent: KeyEvent) -> bool {
        match self.tab1.code_block_mode {
            TuiMode::OutEdit => {
                for KeyMap {
                    keys,
                    r#do,
                    ..
                } in &KEYMAP.keymap
                {
                    if keys.is_empty() || keys[0] != keyevent.into() {
                        continue;
                    }
                    if keys.len() > 1 {
                        self.next_key.store_next(keyevent);
                    }
                    else {
                        self.do_action(r#do).await.unwrap();
                    }
                }
                return true;
            },
            TuiMode::Normal => self
                .tab1
                .normal_map(CrossEvent::Key(keyevent)),
            TuiMode::Insert => self
                .tab1
                .insert_keymap(CrossEvent::Key(keyevent)),
            TuiMode::Select => true,
        };
        true
    }
    async fn tab1_edit_with_editor(&mut self) -> Result<()> {
        let qs_slug = self
            .cur_qs
            .qs_slug
            .clone()
            .unwrap_or_default();
        if qs_slug.is_empty() {
            return Ok(());
        }
        edit(IdSlug::Slug(qs_slug), CodeTestFile::Code).await?;

        self.get_code(&self.cur_qs.clone())
            .await?;

        Ok(())
    }
    async fn handle_key_tab0(&mut self, keyevent: KeyEvent) -> bool {
        if matches!(self.tab0.input_line_mode, TuiMode::Insert) {
            self.tab0
                .keymap_insert(CrossEvent::Key(keyevent));
            return true;
        }

        for KeyMap {
            keys,
            r#do,
            ..
        } in &KEYMAP.keymap
        {
            if keys.is_empty() || keys[0] != keyevent.into() {
                continue;
            }

            if keys.len() > 1 {
                self.next_key.store_next(keyevent);
            }
            else {
                self.do_action(r#do).await.unwrap();
            }
        }
        true
    }
    /// send info for render tui
    pub fn render(&mut self) {
        if let Err(err) = self.tx.send(UserEvent::Render) {
            error!("{err}");
        }
    }
    pub async fn do_action(&mut self, action: &str) -> Result<()> {
        match self.tab_index {
            TabIndex::Tab0 => match action {
                UP => self.tab0.prev_qs(),
                DOWN => self.tab0.next_qs(),
                SYNC_INDEX => self.sync_index(),
                EDIT_IN_TUI => self.tab0.edit(),
                GOTO_EDIT => self.goto_tab(TabIndex::Tab1),
                TOP => self.tab0.first_qs(),
                BOTTOM => self.tab0.last_qs(),
                EDIT_CODE_EDITOR => self
                    .tab0
                    .edit_cur_qs()
                    .await
                    .expect("tab2 edit with editor error"),
                _ => {},
            },
            TabIndex::Tab1 => match action {
                UP => self.tab1.vertical_scroll_k(),
                DOWN => self.tab1.vertical_scroll_j(),
                LEFT => self.tab1.horizontal_scroll_h(),
                RIGHT => self.tab1.horizontal_scroll_l(),
                TOP => self.tab1.vertical_scroll_gg(),
                BOTTOM => self.tab1.vertical_scroll_G(),
                HEAD => self.tab1.pop_head(),
                EDIT_IN_TUI => self.tab1.start_edit_tui(),
                EDIT_CODE_EDITOR => self
                    .tab1_edit_with_editor()
                    .await
                    .expect("tab1 edit with editor error"),
                SUBMIT_CODE => {
                    if self.tab1.show_pop_menu {
                        self.submit_code();
                    }
                },
                TEST_CODE => {
                    if self.tab1.show_pop_menu {
                        self.test_code();
                    }
                },
                TOGGLE_MENU => self.tab1.toggle_menu(),
                TOGGLE_SUBMIT_RES => self.tab1.toggle_submit_res(),
                TOGGLE_TEST_RES => self.tab1.toggle_test_res(),
                ESCAPE => self.tab1.close_pop(),
                _ => {},
            },
            TabIndex::Tab2 => match action {
                UP => self.tab2.up(),
                DOWN => self.tab2.down(),
                EDIT_IN_TUI => self.tab2.enter_input_line(),
                EDIT_CODE_EDITOR => self
                    .tab2
                    .edit_cur_qs()
                    .await
                    .expect("tab2 edit with editor err"),

                TOP => self.tab2.top(),
                BOTTOM => self.tab2.bottom(),

                GOTO_EDIT => self.goto_tab(TabIndex::Tab1),

                PANEL_UP => self.tab2.panel_up(),
                PANEL_DOWN => self.tab2.panel_down(),
                PANEL_LEFT => self.tab2.panel_left(),
                PANEL_RIGHT => self.tab2.panel_right(),

                TOGGLE_CURSOR => self.tab2.toggle_cursor().await,
                SYNC_INDEX => self.sync_new(),
                SAVE_CODE => self.save_code().await?,
                _ => {},
            },
            TabIndex::Tab3 => match action {
                UP => self.tab3.prev_item(),
                DOWN => self.tab3.next_item(),
                TOP => self.tab3.first_item(),
                BOTTOM => self.tab3.last_item(),
                _ => {},
            },
        }
        match action {
            NEXT_TAB => self.next_tab(),
            PREV_TAB => self.prev_tab(),
            EXIT => self.stop().unwrap(),
            _ => {},
        }
        Ok(())
    }
}

impl<'app_lf> App<'app_lf> {
    pub fn stop(&mut self) -> Result<()> {
        self.tx
            .send(UserEvent::Quit)
            .into_diagnostic()
    }
    pub fn sync_index(&mut self) {
        self.tab0.sync_state = true;
        let eve_tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(err) = glob_leetcode()
                .await
                .sync_problem_index()
                .await
            {
                error!("{}", err);
            }

            eve_tx
                .send(UserEvent::SyncDone)
                .unwrap();
        });
    }
    pub fn sync_new(&mut self) {
        self.tab2.sync_state = true;
        let eve_tx = self.tx.clone();
        tokio::spawn(async move {
            if let Err(err) = glob_leetcode()
                .await
                .new_sync_index()
                .await
            {
                error!("{}", err);
            }

            eve_tx
                .send(UserEvent::SyncDoneNew)
                .unwrap();
        });
    }
    pub fn get_qs_detail(&self, idslug: IdSlug, force: bool) {
        let eve_tx = self.tx.clone();
        tokio::spawn(async move {
            let qs = glob_leetcode()
                .await
                .get_qs_detail(idslug, force)
                .await
                .unwrap_or_default();
            eve_tx
                .send(UserEvent::GetQsDone(Box::new(qs)))
                .unwrap();
        });
    }
    pub fn submit_code(&mut self) {
        let id: u32 = self
            .cur_qs
            .question_id
            .parse()
            .unwrap_or_default();
        self.tx
            .send(UserEvent::SubmitCode(id))
            .unwrap();
        self.tab1.submitting = true;
        let eve_tx = self.tx.clone();
        tokio::spawn(async move {
            // min id is 1
            let temp = if id > 0 {
                glob_leetcode()
                    .await
                    .submit_code(IdSlug::Id(id))
                    .await
                    .unwrap_or_default()
            }
            else {
                (SubmitInfo::default(), RunResult::default())
            };
            eve_tx
                .send(UserEvent::SubmitDone(Box::new(temp.1)))
                .unwrap();
        });
    }

    pub fn test_code(&mut self) {
        let id = self
            .cur_qs
            .question_id
            .parse()
            .unwrap_or_default();

        self.tx
            .send(UserEvent::TestCode(id))
            .unwrap();
        self.tab1.submitting = true;

        let eve_tx = self.tx.clone();
        tokio::spawn(async move {
            // min id is 1
            let temp = if id > 0 {
                glob_leetcode()
                    .await
                    .test_code(IdSlug::Id(id))
                    .await
                    .unwrap_or_default()
            }
            else {
                (TestInfo::default(), RunResult::default())
            };
            eve_tx
                .send(UserEvent::TestDone(Box::new(temp.1)))
                .unwrap();
        });
    }
}

// tab1 edit
impl<'app_lf> App<'app_lf> {
    /// from ui to file
    pub async fn save_code(&mut self) -> Result<()> {
        self.save_code = true;
        self.tx
            .send(UserEvent::Render)
            .into_diagnostic()?;

        let lines = self
            .tab1
            .code_block
            .clone()
            .into_lines();
        let chf = CacheFile::new(&IdSlug::Id(self.tab0.current_qs())).await?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(chf.code_path)
            .await
            .into_diagnostic()?;

        for line in lines {
            file.write_all((line + "\n").as_bytes())
                .await
                .into_diagnostic()?;
        }
        file.sync_all()
            .await
            .into_diagnostic()?;

        self.save_code = false;

        Ok(())
    }
    /// from file to ui
    /// # Error:
    /// get qs error (when qs default)
    pub async fn get_code(&mut self, qs: &Question) -> Result<()> {
        if qs.qs_slug.is_none() {
            return Ok(());
        }

        self.tab1.code_block = TextArea::default();

        let chf = CacheFile::new(&IdSlug::Slug(qs.qs_slug.clone().unwrap())).await?;
        if !chf.code_path.exists() {
            glob_leetcode()
                .await
                .get_qs_detail(IdSlug::Slug(qs.qs_slug.clone().unwrap()), false)
                .await?;
        }

        let code = File::open(chf.code_path)
            .await
            .into_diagnostic()?;
        let reader = BufReader::new(code);
        let mut lines = reader.lines();
        while let Some(line) = lines
            .next_line()
            .await
            .into_diagnostic()?
        {
            self.tab1
                .code_block
                .insert_str(line);
            self.tab1
                .code_block
                .insert_newline();
        }
        self.tab1
            .code_block
            .delete_newline();

        Ok(())
    }

    pub async fn get_qs_done(&mut self, qs: Question) {
        match self.get_code(&qs).await {
            // if error, don't update question info
            Ok(()) => {
                self.cur_qs = qs;

                let text = self.cur_qs.to_tui_vec();

                self.tab1.vertical_row_len = text.len();
                self.tab1.vertical_scroll_state = self
                    .tab1
                    .vertical_scroll_state
                    .content_length(text.len());
            },
            Err(err) => error!("{}", err),
        };
        self.render();
    }
}

// base
impl<'app_lf> App<'app_lf> {
    pub async fn new(tx: mpsc::UnboundedSender<UserEvent>) -> App<'app_lf> {
        let tab0 = tab0::SelectQS::new().await;
        let tab1 = tab1::EditCode::new();
        let tab2 = tab2::TopicTagsQS::new().await;
        let tab3 = tab3::KeyMaps::new();
        Self {
            titles: vec![
                "select question",
                "edit",
                "filter with topic",
                "keymaps",
                "config",
            ],
            tab_index: TabIndex::Tab0,

            tx,

            tab0,
            tab1,
            tab2,
            tab3,

            cur_qs: Question::default(),

            pop_temp: false,
            temp_str: String::new(),

            save_code: false,
            next_key: NextKey {
                keys:  Vec::new(),
                times: 0,
            },
        }
    }
    pub fn next_tab(&mut self) {
        self.tab_index.next();
    }
    pub fn prev_tab(&mut self) {
        self.tab_index.prev();
    }
    pub fn goto_tab(&mut self, index: TabIndex) {
        if matches!(index, TabIndex::Tab1) {
            match self.tab_index {
                TabIndex::Tab0 => self.get_qs_detail(IdSlug::Id(self.tab0.current_qs()), false),
                TabIndex::Tab2 => {
                    let qs_slug = self.tab2.cur_qs_slug();
                    if let Some(slug) = qs_slug {
                        self.get_qs_detail(IdSlug::Slug(slug), false);
                    }
                },
                _ => {},
            }
        }
        self.tab_index = index;
    }
}
