// src/cscl_compiler.rs

use crate::coop_vm::Opcode;

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    True,
    False,
    If,
    Else,
    While,
    Function,
    Return,
    Vote,
    AllocateResource,
    UpdateReputation,
    CreateProposal,
    GetProposalStatus,
    Emit,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Equals,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    DoubleEquals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanEquals,
    LessThanEquals,
    And,
    Or,
    Not,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return None;
        }

        match self.input[self.position] {
            '(' => { self.position += 1; Some(Token::LParen) },
            ')' => { self.position += 1; Some(Token::RParen) },
            '{' => { self.position += 1; Some(Token::LBrace) },
            '}' => { self.position += 1; Some(Token::RBrace) },
            ';' => { self.position += 1; Some(Token::Semicolon) },
            ',' => { self.position += 1; Some(Token::Comma) },
            '+' => { self.position += 1; Some(Token::Plus) },
            '-' => { self.position += 1; Some(Token::Minus) },
            '*' => { self.position += 1; Some(Token::Multiply) },
            '/' => { self.position += 1; Some(Token::Divide) },
            '%' => { self.position += 1; Some(Token::Modulo) },
            '=' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::DoubleEquals)
                } else {
                    self.position += 1;
                    Some(Token::Equals)
                }
            },
            '!' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::NotEquals)
                } else {
                    self.position += 1;
                    Some(Token::Not)
                }
            },
            '>' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::GreaterThanEquals)
                } else {
                    self.position += 1;
                    Some(Token::GreaterThan)
                }
            },
            '<' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::LessThanEquals)
                } else {
                    self.position += 1;
                    Some(Token::LessThan)
                }
            },
            '&' => {
                if self.peek_next() == Some('&') {
                    self.position += 2;
                    Some(Token::And)
                } else {
                    None // Invalid token
                }
            },
            '|' => {
                if self.peek_next() == Some('|') {
                    self.position += 2;
                    Some(Token::Or)
                } else {
                    None // Invalid token
                }
            },
            '"' => Some(self.read_string()),
            c if c.is_alphabetic() => Some(self.read_identifier()),
            c if c.is_digit(10) => Some(self.read_number()),
            _ => None, // Invalid token
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn read_string(&mut self) -> Token {
        self.position += 1; // Skip opening quote
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        let value: String = self.input[start..self.position].iter().collect();
        self.position += 1; // Skip closing quote
        Token::String(value)
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        let value: String = self.input[start..self.position].iter().collect();
        match value.as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "function" => Token::Function,
            "return" => Token::Return,
            "vote" => Token::Vote,
            "allocate_resource" => Token::AllocateResource,
            "update_reputation" => Token::UpdateReputation,
            "create_proposal" => Token::CreateProposal,
            "get_proposal_status" => Token::GetProposalStatus,
            "emit" => Token::Emit,
            _ => Token::Identifier(value),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        let mut is_float = false;
        while self.position < self.input.len() && (self.input[self.position].is_digit(10) || self.input[self.position] == '.') {
            if self.input[self.position] == '.' {
                is_float = true;
            }
            self.position += 1;
        }
        let value: String = self.input[start..self.position].iter().collect();
        if is_float {
            Token::Float(value.parse().unwrap())
        } else {
            Token::Integer(value.parse().unwrap())
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn parse(&mut self) -> Vec<Opcode> {
        let mut opcodes = Vec::new();
        while self.position < self.tokens.len() {
            opcodes.append(&mut self.parse_statement());
        }
        opcodes
    }

    fn parse_statement(&mut self) -> Vec<Opcode> {
        match self.current_token() {
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::Function) => self.parse_function_definition(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::Identifier(_)) => self.parse_assignment_or_function_call(),
            Some(Token::Vote) => self.parse_vote_statement(),
            Some(Token::AllocateResource) => self.parse_allocate_resource_statement(),
            Some(Token::UpdateReputation) => self.parse_update_reputation_statement(),
            Some(Token::CreateProposal) => self.parse_create_proposal_statement(),
            Some(Token::GetProposalStatus) => self.parse_get_proposal_status_statement(),
            Some(Token::Emit) => self.parse_emit_statement(),
            _ => Vec::new(), // Error handling should be added here
        }
    }

    fn parse_if_statement(&mut self) -> Vec<Opcode> {
        // Implementation for parsing if statements
        Vec::new()
    }

    fn parse_while_statement(&mut self) -> Vec<Opcode> {
        // Implementation for parsing while loops
        Vec::new()
    }

    fn parse_function_definition(&mut self) -> Vec<Opcode> {
        // Implementation for parsing function definitions
        Vec::new()
    }

    fn parse_return_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::Return);
        let mut opcodes = self.parse_expression();
        opcodes.push(Opcode::Return);
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_assignment_or_function_call(&mut self) -> Vec<Opcode> {
        let identifier = self.consume_identifier();
        match self.current_token() {
            Some(Token::Equals) => self.parse_assignment(identifier),
            Some(Token::LParen) => self.parse_function_call(identifier),
            _ => Vec::new(), // Error handling should be added here
        }
    }

    fn parse_assignment(&mut self, identifier: String) -> Vec<Opcode> {
        self.consume_token(Token::Equals);
        let mut opcodes = self.parse_expression();
        opcodes.push(Opcode::Store(identifier));
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_function_call(&mut self, identifier: String) -> Vec<Opcode> {
        self.consume_token(Token::LParen);
        let mut opcodes = Vec::new();
        while !matches!(self.current_token(), Some(Token::RParen)) {
            opcodes.append(&mut self.parse_expression());
            if matches!(self.current_token(), Some(Token::Comma)) {
                self.consume_token(Token::Comma);
            }
        }
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::Call(identifier));
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_vote_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::Vote);
        self.consume_token(Token::LParen);
        let proposal_id = self.consume_string();
        self.consume_token(Token::Comma);
        let mut opcodes = self.parse_expression(); // This should push a boolean onto the stack
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::Vote(proposal_id));
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_allocate_resource_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::AllocateResource);
        self.consume_token(Token::LParen);
        let resource_id = self.consume_string();
        self.consume_token(Token::Comma);
        let mut opcodes = self.parse_expression(); // This should push an integer onto the stack
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::AllocateResource(resource_id));
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_update_reputation_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::UpdateReputation);
        self.consume_token(Token::LParen);
        let address = self.consume_string();
        self.consume_token(Token::Comma);
        let mut opcodes = self.parse_expression(); // This should push an integer onto the stack
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::UpdateReputation(address));
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_create_proposal_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::CreateProposal);
        self.consume_token(Token::LParen);
        let mut opcodes = self.parse_expression(); // This should push a string onto the stack
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::CreateProposal);
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_get_proposal_status_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::GetProposalStatus);
        self.consume_token(Token::LParen);
        let mut opcodes = self.parse_expression(); // This should push a string onto the stack
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::GetProposalStatus);
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_emit_statement(&mut self) -> Vec<Opcode> {
        self.consume_token(Token::Emit);
        self.consume_token(Token::LParen);
        let event_name = self.consume_string();
        self.consume_token(Token::Comma);
        let mut opcodes = self.parse_expression(); // This should push the event data onto the stack
        self.consume_token(Token::RParen);
        opcodes.push(Opcode::Emit(event_name));
        self.consume_token(Token::Semicolon);
        opcodes
    }

    fn parse_expression(&mut self) -> Vec<Opcode> {
        // Implementation for parsing expressions
        // This should handle arithmetic, logical operations, function calls, etc.
        Vec::new()
    }

    fn consume_token(&mut self, expected: Token) {
        if self.current_token() == Some(&expected) {
            self.position += 1;
        } else {
            panic!("Unexpected token: expected {:?}, found {:?}", expected, self.current_token());
        }
    }

    fn consume_identifier(&mut self) -> String {
        if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.position += 1;
            name
        } else {
            panic!("Expected identifier, found {:?}", self.current_token());
        }
    }

    fn consume_string(&mut self) -> String {
        if let Some(Token::String(value)) = self.current_token().cloned() {
            self.position += 1;
            value
        } else {
            panic!("Expected string, found {:?}", self.current_token());
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
}

pub struct CSCLCompiler {
    lexer: Lexer,
    parser: Parser,
}

pub struct CSCLCompiler {
    lexer: Lexer,
    parser: Parser,
}

impl CSCLCompiler {
    pub fn new(input: &str) -> Self {
        let lexer = Lexer::new(input);
        let tokens = lexer.tokens();
        let parser = Parser::new(tokens);
        CSCLCompiler { lexer, parser }
    }

    pub fn compile(&mut self) -> Vec<Opcode> {
        self.parser.parse()
    }
}

impl Lexer {
    fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "function test(x, y) { return x + y; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokens();
        
        assert_eq!(tokens, vec![
            Token::Function,
            Token::Identifier("test".to_string()),
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ]);
    }

    #[test]
    fn test_compiler() {
        let input = "x = 5 + 3; vote(\"proposal1\", true);";
        let mut compiler = CSCLCompiler::new(input);
        let opcodes = compiler.compile();

        // Note: The exact opcodes will depend on your Opcode enum implementation
        // This is a simplified assertion
        assert!(opcodes.len() > 0);
        // You might want to add more specific assertions based on your Opcode implementation
    }
}