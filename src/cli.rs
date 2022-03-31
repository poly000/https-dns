use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(long, default_value = "127.0.0.1")]
    pub local_address: String,

    #[clap(long, default_value = "53")]
    pub local_port: u16,

    #[clap(long, default_value = "1.1.1.1")]
    pub upstream_address: String,

    #[clap(long, default_value = "443")]
    pub upstream_port: u16,
}
