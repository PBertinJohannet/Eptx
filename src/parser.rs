//! Contains the code for the parser,
//! currently only contains enough to parse expressions and return parse errors.
use token::{TokenType, Token};
use expression::{BinaryExpr, UnaryExpr, LiteralExpr};
use expression::Expr;
use std::fmt;
use statement::{Statement, Declaration, Assignment, IfStatement, FunctionDecl};
use std::collections::HashMap;

#[derive(Debug)]
/// The struct for a parse error, contains just enough information to show
/// the user what happend and where.
pub struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    /// Creates a new parse error.
    pub fn new(token: Token, message: String) -> Self {
        ParseError {
            token: token,
            message: message,
        }
    }
}


impl fmt::Display for ParseError {
    /// Formats the error to a user readable format.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {} : at {:?}:{} line {}",
            self.message,
            self.token.get_type(),
            self.token.get_lexeme(),
            self.token.get_line()
        )
    }
}

/// The parser, contains the tokens and a cursor.
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a new parser from the given tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens,
            current: 0,
        }
    }
    /// parses the given list of token
    ///
    /// If no error occurs, we return the list of the parsed expression.
    ///
    /// If an error occurs, we continue to parse, looking for other errors and return all of them
    /// When an error happens in an expression, we leave the expression to avoid cascading errors.
    ///
    /// But we start again with the new expressions.
    pub fn program(&mut self) -> Result<HashMap<String, FunctionDecl>, Vec<ParseError>> {
        let mut fails = vec![];
        let mut functions = HashMap::new();
        while !self.is_at_end() {
            match self.function_decl() {
                Ok(e) => {
                    let name = e.name().to_string();
                    functions.insert(name, e);
                },
                Err(e) => fails.push(ParseError::new(self.previous(), e)),
            }
            if fails.len() > 5 {
                return Err(fails);
            }
        }

        if fails.is_empty() {
            Ok(functions)
        } else {
            Err(fails)
        }
    }
    /// A new function declaration.
    pub fn function_decl(&mut self) -> Result<FunctionDecl, String>{
        match self.peek().get_type() {
            &TokenType::FUN => {
                self.advance(); // skip the func keyword
                let name = match self.check(&TokenType::IDENTIFIER){
                    true => self.advance().get_lexeme().to_string(),
                    false => return Err("Expected identifier after function".to_string()),
                };
                let arguments = self.func_args()?;
                let scope = self.scope()?;
                Ok(FunctionDecl::new(name, arguments, scope))
            }
            _ => Err("error : expected function declaration there".to_string()),
        }
    }
    /// Parses the declaration of arguments
    /// Should be refactored a little bit tho.
    pub fn func_args(&mut self) -> Result<Vec<String>, String>{
        let mut args = vec![];
        if self.check(&TokenType::LeftParen){
            self.advance();
        } else {
            return Err("Expected left parenthesis after function name".to_string());
        }
        if self.check(&TokenType::IDENTIFIER){
            args.push(self.advance().get_lexeme().to_string());
        } else {
            return Err("Expected identifier".to_string());
        }
        while !self.peek().is_type(&TokenType::RightParen){
            if !self.check(&TokenType::COMMA) {
                return Err("Expected comma after identifier".to_string());
            }
            if self.check(&TokenType::IDENTIFIER){
                args.push(self.advance().get_lexeme().to_string());
            } else {
                return Err("Expected identifier".to_string());
            }
        }
        self.advance();
        Ok(args)
    }

    /// Expects a semeicolon after the statement, else break it.
    pub fn expect_semicolon(&mut self, statement : Statement) -> Result<Statement, String>{
        match self.match_nexts(&[TokenType::SEMICOLON]) {
            true => Ok(statement),
            false => Err("Expected semicolon at the end of statement".to_string()),
        }
    }
    /// parses a statement and waits for a semicolon at the end.
    pub fn statement(&mut self) -> Result<Statement, String> {
        self.break_statement()
    }
    /// try to parse a break statement.
    pub fn break_statement(&mut self)  -> Result<Statement, String> {
        match self.peek().is_type(&TokenType::BREAK){
            true => {
                self.advance();
                self.expect_semicolon(Statement::BreakStatement)
            },
            false => self.if_condition()
        }
    }
    /// parses an if condition.
    pub fn if_condition(&mut self) -> Result<Statement, String> {
        match self.peek().is_type(&TokenType::IF){
            true => self.parse_if(),
            false => self.scope()
        }
    }
    /// Parses all the statements in a scope.
    ///
    /// scopes have implicit semicolons, it will be added if it does not exists.
    pub fn parse_if(&mut self) -> Result<Statement, String> {
        self.advance();
        let condition = self.expression()?;
        let next_statement = self.statement()?;
        // add implicit semicolon.
        Ok(Statement::IfStatement(IfStatement::new(condition, next_statement)))
    }

    /// Parses a scope.
    pub fn scope(&mut self) -> Result<Statement, String> {
        match self.peek().is_type(&TokenType::LeftBrace){
            true => self.parse_scope(),
            false => self.declaration()
        }
    }
    /// Parses all the statements in a scope.
    ///
    /// scopes have implicit semicolons, it will be added if it does not exists.
    pub fn parse_scope(&mut self) -> Result<Statement, String> {
        self.advance();
        let mut statements = vec![];
        while !self.peek().is_type(&TokenType::RightBrace ){
            if self.is_at_end() {
                return Err("Expected closing brace at the end of scope".to_string());
            }
            statements.push(self.statement()?);
        }
        self.advance();
        Ok(Statement::Scope(statements))
    }

    /// Matches a declaration or an assignment.
    pub fn declaration(&mut self) -> Result<Statement, String> {
        let decl = match self.peek().get_type() {
            &TokenType::IDENTIFIER => self.assignment(),
            _ => self.expr_statement(),
        };
        self.expect_semicolon(decl?)
    }
    /// Parses an assignment.
    /// should probably parse
    pub fn assignment(&mut self) -> Result<Statement, String> {
        let ident = self.advance();
        match self.match_nexts(&[TokenType::EQUAL]){
            true => Ok(Statement::Assignment(Assignment::new(ident, self.expression()?))),
            false => Err("expected Equals after variable declaration".to_string()),
        }
    }
    /// Matches an expression statement, an expr ending with a semicolon.
    pub fn expr_statement(&mut self) -> Result<Statement, String>{
        Ok(Statement::ExprStatement(self.expression()?))
    }
    /// Parses an expression, the lowest level of precedence is equality.
    pub fn expression(&mut self) -> Result<Expr, String> {
        self.equality()
    }
    /// Parses an equality by searching fot comparisons.
    pub fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        while self.match_nexts(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let previous = self.previous();
            let right = self.comparison()?;
            let new_expr = Expr::Binary(BinaryExpr::new(expr, previous.clone(), right));
            expr = new_expr;
        }
        Ok(expr)
    }
    /// parses a comparison.
    pub fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.addition()?;
        while self.match_nexts(
            &[
                TokenType::GreaterEqual,
                TokenType::LessEqual,
                TokenType::GREATER,
                TokenType::LESS,
            ],
        )
        {
            let previous = self.previous();
            let right = self.addition()?;
            let new_expr = Expr::Binary(BinaryExpr::new(expr, previous.clone(), right));
            expr = new_expr;
        }
        Ok(expr)
    }
    /// Parses an adition.
    pub fn addition(&mut self) -> Result<Expr, String> {
        let mut expr = self.multiplication()?;
        while self.match_nexts(&[TokenType::MINUS, TokenType::PLUS]) {
            let previous = self.previous();
            let right = self.multiplication()?;
            let new_expr = Expr::Binary(BinaryExpr::new(expr, previous.clone(), right));
            expr = new_expr;
        }
        Ok(expr)
    }

    /// Parses a multiplication.
    pub fn multiplication(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.match_nexts(&[TokenType::STAR, TokenType::SLASH]) {
            let previous = self.previous();
            let right = self.unary()?;
            let new_expr = Expr::Binary(BinaryExpr::new(expr, previous.clone(), right));
            expr = new_expr;
        }
        Ok(expr)
    }

    /// Parses an unary expression.
    pub fn unary(&mut self) -> Result<Expr, String> {
        if self.match_nexts(&[TokenType::MINUS, TokenType::BANG]) {
            let previous = self.previous();
            println!("prev : {:?} ask unary", previous);
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr::new(previous.clone(), right)));
        }
        return self.literal();
    }

    /// Parses a litteral expression.
    pub fn literal(&mut self) -> Result<Expr, String> {
        if self.match_nexts(&[TokenType::LeftParen]) {
            let expr = self.expression();
            match self.peek().is_type(&TokenType::RightParen) {
                true => {
                    self.advance();
                    expr
                }
                false => Err("Expected right parenthesis".to_string()),
            }
        } else {
            let token = self.advance();
            match token.get_type() {
                &TokenType::NUMBER => Ok(Expr::Literal(LiteralExpr::NUMBER(
                    token.get_lexeme().parse::<f64>().unwrap(),
                ))),
                &TokenType::STRING => Ok(Expr::Literal(
                    LiteralExpr::STRING(token.get_lexeme().to_string()),
                )),
                &TokenType::NIL => Ok(Expr::Literal(LiteralExpr::NUMBER(0.0))),
                &TokenType::FALSE => Ok(Expr::Literal(LiteralExpr::NUMBER(0.0))),
                &TokenType::TRUE => Ok(Expr::Literal(LiteralExpr::NUMBER(1.0))),
                &TokenType::IDENTIFIER => Ok(Expr::Identifier(token.get_lexeme().to_string())),
                _ => Err("Cant parse literal".to_string()),
            }
        }
    }
    /// Given a list of token types, matches one of them if possible and consume it
    /// If no matches were found, do nothing.
    pub fn match_nexts(&mut self, tokens: &[TokenType]) -> bool {
        for tp in tokens.iter() {
            if self.check(tp) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    /// Checks that the next token is of the given type.
    pub fn check(&mut self, token_type: &TokenType) -> bool {
        !(self.is_at_end() || !self.peek().is_type(token_type))

    }

    /// Peeks for the next token, without conduming it.
    pub fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Advance and consume the next token, returning it
    pub fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    /// Returns the previous token.
    pub fn previous(&self) -> Token {
        if self.current == 0 {
            self.tokens[0].clone()
        } else {
            self.tokens[self.current - 1].clone()
        }
    }

    /// Checks if we are at the end of the file.
    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}
