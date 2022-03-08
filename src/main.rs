use clap::{ArgEnum, Parser};
use serde::Serialize;

#[derive(Debug, Clone, ArgEnum)]
enum Status {
    Success,
    Failure,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, arg_enum)]
    status: Status,

    #[clap(long)]
    service: String,

    #[clap(long)]
    environment: String,

    #[clap(long)]
    user: String,

    #[clap(long)]
    build_number: usize,

    #[clap(long)]
    build_url: String,

    #[clap(long)]
    git_commit: String,

    #[clap(long)]
    git_message: String,

    #[clap(long)]
    hook_url: String,

    #[clap(long)]
    channel: String,
}

#[derive(Serialize)]
struct Message {
    username: String,
    channel: String,
    blocks: Vec<Block>,
}

#[derive(Serialize)]
struct Block {
    #[serde(rename = "type")]
    block_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextEntry>,

    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<TextEntry>>,
}

impl Block {
    fn text(text: String) -> Self {
        Block {
            block_type: "section".to_string(),
            text: Some(TextEntry {
                entry_type: "mrkdwn".to_string(),
                text,
            }),
            fields: None,
        }
    }

    fn fields(fields: &[String]) -> Self {
        let fields = fields
            .iter()
            .map(|val| TextEntry {
                entry_type: "mrkdwn".to_string(),
                text: val.to_string(),
            })
            .collect::<Vec<TextEntry>>();

        Block {
            block_type: "section".to_string(),
            text: None,
            fields: Some(fields),
        }
    }
}

#[derive(Serialize)]
struct TextEntry {
    #[serde(rename = "type")]
    entry_type: String,
    text: String,
}

fn get_header(args: &Args) -> String {
    match args.status {
        Status::Success => format!(
            ":white_check_mark: Deployment of *{}* to *{}* successful.",
            args.service, args.environment
        ),
        Status::Failure => format!(
            ":no_entry: Deployment of *{}* to *{}* failed.",
            args.service, args.environment
        ),
    }
}

fn get_build(args: &Args) -> String {
    format!("*Build:*\n<{}|{}>", args.build_url, args.build_number)
}

fn get_triggerer(args: &Args) -> String {
    format!("*Triggered by:*\n{}", args.user)
}

fn get_git_info(args: &Args) -> String {
    format!("```Commit: {}\n{}```", args.git_commit, args.git_message)
}

fn main() {
    let args = Args::parse();

    let mut message = Message {
        username: "Bitbucket Pipelines".to_string(),
        channel: args.channel.clone(),
        blocks: Vec::new(),
    };

    message.blocks.push(Block::text(get_header(&args)));
    message
        .blocks
        .push(Block::fields(&[get_build(&args), get_triggerer(&args)]));
    message.blocks.push(Block::text(get_git_info(&args)));

    let resp = ureq::post(&args.hook_url)
        .set("Content-Type", "application/json")
        .send_json(ureq::json!(message));

    if let Err(ureq::Error::Status(_code, response)) = resp {
        eprintln!("Error posting to slack: {:?}", response);
    }
}
