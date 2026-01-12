use insim::identifiers::ClickId;
use tokio::sync::{mpsc, watch};

pub trait Component<Ctx> {
    type Message: Clone + Send + 'static;
    #[allow(unused)]
    fn update(&mut self, msg: Self::Message) {}
    fn render(&self, ctx: &Ctx) -> super::Node<Self::Message>;
}

/// View
pub trait View: Component<Self::GlobalProps> + Sized + Send + 'static {
    type GlobalProps: Clone + Send + Sync + Default + 'static;
    type ConnectionProps: Clone + Send + Sync + Default + 'static;

    /// New!
    fn mount(tx: mpsc::UnboundedSender<Self::Message>) -> Self;

    /// Run the UI
    fn run(
        mut state_rx: watch::Receiver<Self::GlobalProps>,
        mut input_rx: mpsc::UnboundedReceiver<ClickId>,
    ) -> (
        mpsc::UnboundedSender<Self::Message>,
        tokio::task::JoinHandle<()>,
    ) {
        let (internal_tx, mut internal_rx) = mpsc::unbounded_channel();
        let external_tx = internal_tx.clone();

        let handle = tokio::spawn(async move {
            let mut root = Self::mount(internal_tx);

            // TODO: draw

            loop {
                tokio::select! {
                    Ok(_) = state_rx.changed() => {
                        // TODO: draw
                    }

                    // internal messages (i.e. clock ticks in this example)
                    Some(_msg) = internal_rx.recv() => {
                        // TODO: draw
                    }

                    // user input (click ids)
                    Some(_click_id) = input_rx.recv() => {
                        // TODO: draw
                    }
                }
            }
        });

        (external_tx, handle)
    }
}
