use std::{sync::atomic::Ordering, time::Duration};

use leetcode_api::{
    dao::query_all_index,
    glob_leetcode,
    leetcode::{
        CUR_QS_INDEX_NUM, CUR_TOPIC_QS_INDEX_NUM, TOTAL_QS_INDEX_NUM, TOTAL_TOPIC_QS_INDEX_NUM,
    },
};
use tracing::error;

use crate::{
    app::{inner::App, topic::TopicTagsQS},
    mytui::myevent::UserEvent,
};

impl<'app_lf> App<'app_lf> {
    pub fn sync_index(&mut self) -> bool {
        if self.select.sync_state {
            return false;
        }
        self.select.sync_state = true;
        let eve_tx = self.events.tx.clone();

        let handle = tokio::spawn(async move {
            if let Err(err) = glob_leetcode()
                .await
                .sync_problem_index()
                .await
            {
                error!("{}", err);
            }

            eve_tx
                .send(UserEvent::SyncDone)
                .expect("sync index failed");
        });
        let tx = self.events.tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(200)).await;
                let a = f64::from(CUR_QS_INDEX_NUM.load(Ordering::Relaxed))
                    / f64::from(TOTAL_QS_INDEX_NUM.load(Ordering::Relaxed));
                if a <= 1.0 {
                    tx.send(UserEvent::Syncing(a)).ok();
                    tx.send(UserEvent::Render).ok();
                }
                if handle.is_finished() {
                    break;
                }
            }
        });
        true
    }
    /// refresh `all_questions`, `filtered_qs`
    pub async fn sync_done(&mut self) {
        self.select.sync_state = false;
        let questions = query_all_index()
            .await
            .unwrap_or_default();
        self.select.all_questions = questions.into();
        self.select.filter_by_input();

        self.render();
    }
    pub fn sync_new(&mut self) -> bool {
        if self.topic.sync_state {
            return false;
        }

        self.topic.sync_state = true;
        let eve_tx = self.events.tx.clone();
        let handle = tokio::spawn(async move {
            if let Err(err) = glob_leetcode()
                .await
                .sync_index_topic()
                .await
            {
                error!("{}", err);
            }

            eve_tx
                .send(UserEvent::SyncDoneNew)
                .expect("sync_new send failed");
        });
        let tx = self.events.tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(200)).await;
                let a = CUR_TOPIC_QS_INDEX_NUM.load(Ordering::Relaxed) as f64
                    / TOTAL_TOPIC_QS_INDEX_NUM.load(Ordering::Relaxed) as f64;
                if a <= 1.0 {
                    tx.send(UserEvent::SyncingNew(a)).ok();
                    tx.send(UserEvent::Render).ok();
                }
                if handle.is_finished() {
                    break;
                }
            }
        });
        true
    }
    /// refresh `all_topic_qs`, `filtered_qs`, `topic_tags`, `difficultys`
    pub async fn sync_new_done(&mut self) {
        self.topic.sync_state = false;
        let base = TopicTagsQS::base_info().await;
        self.topic.all_topic_qs = base.0;
        self.topic.topic_tags = base.1;
        self.topic.difficultys = base
            .2
            .iter()
            .map(|v| v.0.clone())
            .collect();
        self.topic.ac_status = base.2;

        self.topic
            .refresh_filter_by_topic_diff()
            .await;
        self.topic.refresh_filter_by_input();

        self.render();
    }
}
