use tinytemplate::TinyTemplate;
use tracing::error;

use crate::client::BiliMessage;

use super::DanmujiPlugin;

pub struct GiftThanker {
    // thank_msg_queue: VecDeque<String>,
    template: String,
}

impl GiftThanker {
    pub fn new(template: &str) -> Self {
        Self {
            // thank_msg_queue: VecDeque::new(),
            template: template.to_string(),
        }
    }
}
impl DanmujiPlugin for GiftThanker {
    fn process_mesage(&mut self, msg: &BiliMessage) -> Option<String> {
        match msg {
            BiliMessage::Gift(ref gift) => {
                let mut template = TinyTemplate::new();
                if let Err(err) = template.add_template("gift", &self.template) {
                    error!("Invalid Gift Thank Template: {}", err);
                    None
                } else if let Ok(res) = template.render("gift", gift) {
                    Some(res)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::GiftMessage;

    #[tokio::test]
    async fn test_template_basics() {
        let test_msg = BiliMessage::Gift(GiftMessage::default_message());
        let mut thanker = GiftThanker::new("感谢{uname}投喂的{gift_num}个{gift_name}");
        assert_eq!(
            Some("感谢测试用户投喂的1个小花花".to_string()),
            thanker.process_mesage(&test_msg)
        );
    }
}
