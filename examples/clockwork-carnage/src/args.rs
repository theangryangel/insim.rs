use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct InSimArgs {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    pub addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    pub admin_password: Option<String>,
}
