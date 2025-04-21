use std::collections::{HashSet, VecDeque};

use logos::Logos;

#[derive(Debug, Clone, Logos, PartialEq)]
pub enum MessageToken {
    #[regex("(```\n)|`")]
    CodeblockMarker,
    #[regex("<@[0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26}>", |lex| lex.slice()[2..lex.slice().len() - 1].to_owned())]
    UserMention(String),
    #[regex("<%[0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26}>", |lex| lex.slice()[2..lex.slice().len() - 1].to_owned())]
    RoleMention(String),
    #[token("@everyone")]
    MentionEveryone,
    #[token("@online")]
    MentionOnline
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MessageResults {
    pub user_mentions: HashSet<String>,
    pub role_mentions: HashSet<String>,
    pub mentions_everyone: bool,
    pub mentions_online: bool
}

struct MessageParserIterator<I> {
    inner: I,
    temp: VecDeque<MessageToken>
}

impl<I: Iterator<Item = MessageToken>> Iterator for MessageParserIterator<I> {
    type Item = MessageToken;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.temp.is_empty() {
            self.temp.pop_front()
        } else {
            let token = self.inner.next();

            if let Some(token) = token {
                if token == MessageToken::CodeblockMarker {
                    loop {
                        let next_token = self.inner.next();

                        if next_token == Some(MessageToken::CodeblockMarker) {
                            self.temp.clear();
                            return next_token
                        } else if next_token.is_none()  {
                            return Some(MessageToken::CodeblockMarker)
                        } else if let Some(token) = next_token {
                            self.temp.push_back(token);
                        }
                    }

                } else {
                    Some(token)
                }
            } else {
                None
            }
        }
    }
}

pub fn parse_message_iter(text: &str) -> impl Iterator<Item = MessageToken> + '_ {
    MessageParserIterator {
        inner: MessageToken::lexer(text).flatten(),
        temp: VecDeque::new()
    }
}

pub fn parse_message(text: &str) -> MessageResults {
    let mut results = MessageResults::default();

    for token in parse_message_iter(text) {
        match token {
            MessageToken::CodeblockMarker => {},
            MessageToken::UserMention(id) => { results.user_mentions.insert(id); },
            MessageToken::RoleMention(id) => { results.role_mentions.insert(id); },
            MessageToken::MentionEveryone => results.mentions_everyone = true,
            MessageToken::MentionOnline => results.mentions_online = true,
        };
    };

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_nodes() {
        let output = parse_message_iter("Hello everyone").collect::<Vec<_>>();

        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_simple_user_mention() {
        let output = parse_message_iter("Hello <@01FD58YK5W7QRV5H3D64KTQYX3>.").collect::<Vec<_>>();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], MessageToken::UserMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
    }

    #[test]
    fn test_simple_role_mention() {
        let output = parse_message_iter("Hello <%01FD58YK5W7QRV5H3D64KTQYX3>.").collect::<Vec<_>>();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], MessageToken::RoleMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
    }

    #[test]
    fn test_mention_everyone() {
        let output = parse_message_iter("Hello @everyone.").collect::<Vec<_>>();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], MessageToken::MentionEveryone);
    }

    #[test]
    fn test_mention_online() {
        let output = parse_message_iter("Hello @online.").collect::<Vec<_>>();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], MessageToken::MentionOnline);
    }

    #[test]
    fn test_everything() {
        let output = parse_message_iter("Hello <@01FD58YK5W7QRV5H3D64KTQYX3>, <%01FD58YK5W7QRV5H3D64KTQYX3>, @everyone and @online.").collect::<Vec<_>>();

        assert_eq!(output.len(), 4);
        assert_eq!(output[0], MessageToken::UserMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[1], MessageToken::RoleMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[2], MessageToken::MentionEveryone);
        assert_eq!(output[3], MessageToken::MentionOnline);
    }

    #[test]
    fn test_everything_no_spaces() {
        let output = parse_message_iter("<@01FD58YK5W7QRV5H3D64KTQYX3><%01FD58YK5W7QRV5H3D64KTQYX3>@everyone@online").collect::<Vec<_>>();

        assert_eq!(output.len(), 4);
        assert_eq!(output[0], MessageToken::UserMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[1], MessageToken::RoleMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[2], MessageToken::MentionEveryone);
        assert_eq!(output[3], MessageToken::MentionOnline);
    }

    #[test]
    fn test_codeblock_no_mentions() {
        let output = parse_message_iter("```\n<@01FD58YK5W7QRV5H3D64KTQYX3><%01FD58YK5W7QRV5H3D64KTQYX3>@everyone@online\n```").collect::<Vec<_>>();

        assert_eq!(output.len(), 2);
        assert_eq!(output[0], MessageToken::CodeblockMarker);
        assert_eq!(output[1], MessageToken::CodeblockMarker);
    }

    #[test]
    fn test_uncontained_codeblock_should_mention() {
        let output = parse_message_iter("```\n<@01FD58YK5W7QRV5H3D64KTQYX3><%01FD58YK5W7QRV5H3D64KTQYX3>@everyone@online").collect::<Vec<_>>();

        assert_eq!(output.len(), 5);
        assert_eq!(output[0], MessageToken::CodeblockMarker);
        assert_eq!(output[1], MessageToken::UserMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[2], MessageToken::RoleMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[3], MessageToken::MentionEveryone);
        assert_eq!(output[4], MessageToken::MentionOnline);
    }

    #[test]
    fn test_inline_codeblock_no_mentions() {

        let output = parse_message_iter("`<@01FD58YK5W7QRV5H3D64KTQYX3><%01FD58YK5W7QRV5H3D64KTQYX3>@everyone@online`").collect::<Vec<_>>();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], MessageToken::CodeblockMarker);
    }

    #[test]
    fn test_uncontained_inline_codeblock_should_mention() {
        let output = parse_message_iter("`text`<@01FD58YK5W7QRV5H3D64KTQYX3><%01FD58YK5W7QRV5H3D64KTQYX3>@everyone@online").collect::<Vec<_>>();

        assert_eq!(output.len(), 5);
        assert_eq!(output[0], MessageToken::CodeblockMarker);
        assert_eq!(output[1], MessageToken::UserMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[2], MessageToken::RoleMention("01FD58YK5W7QRV5H3D64KTQYX3".to_string()));
        assert_eq!(output[3], MessageToken::MentionEveryone);
        assert_eq!(output[4], MessageToken::MentionOnline);
    }
}