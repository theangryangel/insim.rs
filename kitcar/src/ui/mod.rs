//! An implementation of a retained immediate mode UI.
//! A hybrid approach that combines the programming model of immediate mode with the
//! performance optimizations of retained mode.
//! Functions are called every frame, but we diff the output to minimise the rendering
//! requirements.
//! Each plugin will be responsible for it's own set of Ui's. Nothing shared except for the id_pool.
//! `Ui` represents the ui for a single connection.
pub mod components;
pub mod id_pool;
pub mod styled;
pub mod ui;
pub mod vdom;

pub use id_pool::ClickIdPool;
pub use styled::Styled;
pub use ui::{Ui, UiDiff};
pub use vdom::Element;

#[cfg(test)]
mod test {
    use insim::{identifiers::ConnectionId, insim::BtnStyle};

    use super::*;

    #[derive(Clone, PartialEq, Default)]
    pub struct AppProps {
        pub empty: bool,
        pub bar: bool,
    }

    fn app(props: &AppProps) -> Option<Element> {
        if props.empty {
            return None;
        }

        let mut children = Vec::new();

        children.push(
            Element::Button {
                text: "foo".to_string(),
                key: "1".into(),
                style: taffy::Style::DEFAULT,
                btnstyle: BtnStyle::default(),
            }
            .w(5.0)
            .h(5.0),
        );

        if props.bar {
            children.push(Element::Button {
                text: "bar".to_string(),
                key: "2".into(),
                style: taffy::Style::DEFAULT,
                btnstyle: BtnStyle::default(),
            });
        }

        Some(Element::Container {
            children,
            style: taffy::Style::DEFAULT,
        })
    }

    #[test]
    fn test_ui() {
        let mut ui = Ui::new(ClickIdPool::new(), ConnectionId::ALL, app);

        let diff = ui
            .render(&AppProps {
                empty: false,
                bar: false,
            })
            .expect("Initial render should render *something*");

        assert_eq!(diff.to_update.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);

        let expected_click_id = diff.to_update[0].clickid;

        assert_eq!(ui.key_to_click_id("1"), Some(&expected_click_id));

        assert_eq!(diff.to_update[0].text, "foo");

        let diff = ui.render(&AppProps {
            empty: false,
            bar: false,
        });

        // nothing changed
        assert!(diff.is_none());

        assert_eq!(ui.key_to_click_id("1"), Some(&expected_click_id));

        let diff = ui
            .render(&AppProps {
                empty: false,
                bar: true,
            })
            .expect("when updating bar, we should get a diff");

        assert_eq!(diff.to_update.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);

        assert_eq!(diff.to_update[0].text, "bar");
        assert_ne!(diff.to_update[0].clickid, expected_click_id); // we dont reuse an id

        let diff = ui
            .render(&AppProps {
                empty: true,
                bar: true,
            })
            .expect("when updating bar, we should get a diff");

        assert_eq!(diff.to_remove.len(), 2, "received diff: {:?}", diff);

        assert_eq!(ui.key_to_click_id("1"), None);
    }
}
