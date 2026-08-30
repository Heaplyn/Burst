#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast;
use ast::*;
use std::collections::*;
use std::iter::Map;
use std::task::Context;
use config::*;
use lexer::*;
use parser::*;
use elaboration::*;

// ============================================
// Compiler Result Type
// ============================================

pub type CompilerResult<T> = Result<T, CompilerError>;


fn FormatValue(v: &Value) -> String { 
    match v { 
        Value::Int(n) => n.to_string(), 
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        _ => "...".to_string(),
    } 
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerError {
    LexerError(String),
    ParserError(String),
    ElaborationError(String),
    TypeError(String),
    RuntimeError(String),
    InternalError(String),
}

// ============================================
// Code Runner
// ============================================
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompilerConfig {
    
}
impl CompilerConfig {
    pub fn New() -> Self {
        Self {
        }
    }
}
/// signature every built-in function shares: takes the runner (for state/output)
/// and the already-evaluated arguments, returns a Value.
pub type BuiltinFn = fn(&mut CodeRunner, &[Value]) -> CompilerResult<Value>;

#[derive(Debug, Clone)]
pub struct CodeRunner {
    Context: ExecutionContext,
    Trace: LayerTrace,
    Config: CompilerConfig,
    /// str function name -> built-in implementation
    Builtins: HashMap<String, BuiltinFn>,
}
pub fn AddBuiltins(Map: &mut HashMap<String, BuiltinFn>) {
    Map.insert("print".to_string(), Builtin_Print as BuiltinFn);
    Map.insert("println".to_string(), Builtin_Println as BuiltinFn);    
}
impl CodeRunner {
    pub fn New(Config: CompilerConfig) -> Self {
        let mut Builtins: HashMap<String, BuiltinFn> = HashMap::new();
        AddBuiltins(&mut Builtins);

        Self {
            Context: ExecutionContext::New(),
            Trace: LayerTrace::New(),
            Config,
            Builtins,
        }
    }

    // ============================================
    // Expression Evaluation
    // ============================================

    pub fn EvaluateExpression(&mut self, Expr: &Expression) -> CompilerResult<Value> {
        match Expr {
            Expression::LiteralInt(Val) => Ok(Value::Int(*Val)),
            Expression::LiteralFloat(Val) => Ok(Value::Float(*Val)),
            Expression::LiteralBool(Val) => Ok(Value::Bool(*Val)),
            Expression::LiteralString(Val) => Ok(Value::String(Val.clone())),
            Expression::Variable(Name) => {
                self.Context
                    .GetVariable(Name)
                    .ok_or_else(|| CompilerError::RuntimeError(format!("Variable '{}' not found", Name)))
            }
            Expression::BinaryOp { Op, Lhs, Rhs } => {
                let LeftVal = self.EvaluateExpression(Lhs)?;
                let RightVal = self.EvaluateExpression(Rhs)?;
                self.EvaluateBinaryOp(Op, LeftVal, RightVal)
            }
            Expression::FunctionCall { Name, Args } => {
                // 1. Evaluate every argument in the caller's scope.
                let mut ArgValues = Vec::with_capacity(Args.len());
                for A in Args {
                    ArgValues.push(self.EvaluateExpression(A)?);
                }
                // 2. Built-in first: if the name is in the Builtins map, run it and return.
                if let Some(Result) = self.CallBuiltin(Name, &ArgValues) {
                    return Result;
                }

                // 3. Otherwise resolve a user function by name, cloning so the immutable
                //    borrow on self.Trace.Layers ends before we run &mut self.
                let FoundFunc: Layer = self
                    .FindFunctionByName(&self.Trace.Layers, Name)?
                    .clone();

                // 4. Push a fresh frame, bind parameters to argument values, run the body.
                self.Context.PushFrame(Name);
                if let LayerKind::Function { Params: DeclParams, .. } = &FoundFunc.Kind {
                    if DeclParams.len() != ArgValues.len() {
                        self.Context.PopFrame();
                        return Err(CompilerError::RuntimeError(format!(
                            "Function '{}' expected {} arg(s), got {}",
                            Name, DeclParams.len(), ArgValues.len()
                        )));
                    }
                    for (P, V) in DeclParams.iter().zip(ArgValues.into_iter()) {
                        self.Context.SetVariable(&P.Name, V, false);
                    }
                }
                let Result = self.RunBlock(&FoundFunc.Children)?;
                self.Context.PopFrame();
                Ok(Result)
            }
            _ => Err(CompilerError::RuntimeError("Unsupported expression".to_string())),
        }
    }

    fn EvaluateBinaryOp(&mut self, Op: &str, Lhs: Value, Rhs: Value) -> CompilerResult<Value> {
        match (Op, Lhs, Rhs) {
            ("+", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Int(Lhs + Rhs)),
            ("-", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Int(Lhs - Rhs)),
            ("*", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Int(Lhs * Rhs)),
            ("/", Value::Int(Lhs), Value::Int(Rhs)) => {
                if Rhs == 0 {
                    Err(CompilerError::RuntimeError("Division by zero".to_string()))
                } else {
                    Ok(Value::Int(Lhs / Rhs))
                }
            }
            ("+", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Float(Lhs + Rhs)),
            _ => Err(CompilerError::RuntimeError(format!("Invalid binary operation: {}", Op))),
        }
    }

    // ============================================
    // Layer Execution
    // ============================================

    pub fn RunLayer(&mut self, Layer: &Layer) -> CompilerResult<Value> {
        match &Layer.Kind {
            LayerKind::Program => {
                for Child in &Layer.Children {
                    self.RunLayer(Child)?;
                }
                Ok(Value::Unit)
            }

            LayerKind::Function { Name, Params, ReturnType, .. } => {
                // Push function context
                self.Context.PushFrame(Name);
                
                // Execute function body
                let Result = self.RunBlock(&Layer.Children)?;
                
                // Pop context
                self.Context.PopFrame();
                
                // Check return type
                if let Some(ReturnType) = ReturnType {
                    if !self.CheckType(&Result, ReturnType) {
                        return Err(CompilerError::TypeError(
                            format!("Return type mismatch in function '{}'", Name)
                        ));
                    }
                }
                
                Ok(Result)
            }
            // NOTE: LayerKind::FunctionCall is never emitted by the parser — calls arrive
            // as Expression::FunctionCall and are handled in EvaluateExpression above.

            // A bare expression statement (e.g. `add(3, 4);` or `2 + 5 * 3;`) evaluates
            // to its value, so the top level runs as a sequence of expressions.
            LayerKind::Expression(Expr) => self.EvaluateExpression(Expr),

            LayerKind::VariableBinding { Name, TypeAnnotation, IsMutable, Hooks, InitialValue } => {
                // Evaluate initial value
                let mut Value = if let Some(Expr) = InitialValue {
                    self.EvaluateExpression(Expr)?
                } else {
                    // Use default value based on type
                    self.DefaultValue(TypeAnnotation)?
                };

                // Run on_assign hooks
                for Hook in Hooks {
                    if Hook.Kind != HookKind::OnAssign { continue; }
                    for child_kind in &Hook.Body {
                        if let LayerKind::Function { Name, Params, ReturnType, .. } = child_kind {
                            // Create a temporary Layer to run the function
                            let temp_func_layer = Layer {
                                Id: LayerId { Id: 0 }, // Placeholder ID
                                Kind: child_kind.clone(),
                                Metadata: LayerMetadata {
                                    SourceLocation: SourceLocation::Builtin(),
                                    Docs: None,
                                    Directives: Vec::new(),
                                    Optimization: OptimizationHints {
                                        AggressiveLoopFolding: false,
                                        TraceObservability: ObservabilityMode::Strict,
                                        RegisterPressure: RegisterPressureMode::Auto,
                                        InlineThreshold: None,
                                    },
                                    Custom: std::collections::HashMap::new(),
                                },
                                Children: vec![], // The body statements
                                Constraints: vec![],
                                Observability: ObservabilityFlags {
                                    ObservableValues: Vec::new(),
                                    AffectsOutput: true,
                                    AffectsHardware: true,
                                    ObservableToTrace: true,
                                },
                                TypeStorage: TypeStorage::default(),
                                TraceInfo: TraceInfo {
                                    TraceId: "unknown".to_string(),
                                    Depth: 0,
                                    Context: ast::TraceContext::Root,
                                    TypeEnv: TypeStorage::default(),
                                },
                            };
                            Value = self.RunLayer(&temp_func_layer)?;
                        }
                    }
                }

                // Store variable
                self.Context.SetVariable(Name, Value.clone(), *IsMutable);

                // Run on_change hooks
                for Hook in Hooks {
                    if Hook.Kind != HookKind::OnChange { continue; }
                    for child_kind in &Hook.Body {
                        if let LayerKind::Function { Name, Params, ReturnType, .. } = child_kind {
                            // Create a temporary Layer to run the function
                            let temp_func_layer = Layer {
                                Id: LayerId { Id: 0 }, // Placeholder ID
                                Kind: child_kind.clone(),
                                Metadata: LayerMetadata {
                                    SourceLocation: SourceLocation::Builtin(),
                                    Docs: None,
                                    Directives: Vec::new(),
                                    Optimization: OptimizationHints {
                                        AggressiveLoopFolding: false,
                                        TraceObservability: ObservabilityMode::Strict,
                                        RegisterPressure: RegisterPressureMode::Auto,
                                        InlineThreshold: None,
                                    },
                                    Custom: std::collections::HashMap::new(),
                                },
                                Children: vec![], // The body statements
                                Constraints: vec![],
                                Observability: ObservabilityFlags {
                                    ObservableValues: Vec::new(),
                                    AffectsOutput: true,
                                    AffectsHardware: true,
                                    ObservableToTrace: true,
                                },
                                TypeStorage: TypeStorage::default(),
                                TraceInfo: TraceInfo {
                                    TraceId: "unknown".to_string(),
                                    Depth: 0,
                                    Context: ast::TraceContext::Root,
                                    TypeEnv: TypeStorage::default(),
                                },
                            };
                            self.RunLayer(&temp_func_layer)?;
                        }
                    }
                }

                Ok(Value::Unit)
            }

            /*
            LayerKind::Havoc { Target, KeepCache } => {
                // Invalidate cache for target
                self.Context.Invalidate(Target);
                
                // Keep specified values in cache
                for Keep in KeepCache {
                    self.Context.KeepCache(Keep);
                }
                
                // Record observability
                self.Trace.RecordEvent(TraceEvent::Havoc {
                    Target: Target.to_string(),
                });
                
                Ok(Value::Unit)
            }

            
             */
            LayerKind::Return { Value: Expr } => {
                if let Some(Expr) = Expr {
                    let Val = self.EvaluateExpression(Expr)?;
                    Ok(Val)
                } else {
                    Ok(Value::Unit)
                }
            }

            _ => Err(CompilerError::RuntimeError(format!(
                "Unsupported layer kind: {:?}",
                Layer.Kind
            ))),
        }
    }

    fn RunBlock(&mut self, Children: &[Layer]) -> CompilerResult<Value> {
        let mut Result = Value::Unit;
        
        for Child in Children {
            Result = self.RunLayer(Child)?;
            
            // Check if we hit a return
            if let LayerKind::Return { .. } = &Child.Kind {
                break;
            }
        }
        
        Ok(Result)
    }

    // ============================================
    // Code Entry Point
    // ============================================

    pub fn RunCode(&mut self, Layers: &[Layer]) -> CompilerResult<Value> {
        // The driver hands us the single `Program` layer wrapped in a slice, so
        // unwrap it to reach the real top-level items (functions, globals, calls).
        let TopLevel: &[Layer] = match Layers {
            [Program] if matches!(Program.Kind, LayerKind::Program) => &Program.Children,
            other => other,
        };

        // Snapshot the top-level items so FunctionCall can resolve names later.
        self.Trace = LayerTrace::NewFrom(TopLevel);

        // Run the top level like a script: execute every item in order, but skip
        // function *definitions* — those are only entered when they're called.
        // No `main` is required.
        let mut Last = Value::Unit;
        for Item in TopLevel {
            if matches!(Item.Kind, LayerKind::Function { .. }) {
                continue;
            }
            Last = self.RunLayer(Item)?;
        }
        Ok(Last)
    }

    fn FindMainFunction<'lifetime>(&self, Layers: &'lifetime [Layer]) -> CompilerResult<&'lifetime Layer> {
        for Layer in Layers {
            if let LayerKind::Function { Name, .. } = &Layer.Kind {
                if Name == "main" {
                    return Ok(Layer);
                }
            }
        }
        Err(CompilerError::RuntimeError("No 'main' function found".to_string()))
    }

    //Finding any func under what name
    fn FindFunctionByName<'lifetime>(&self,Layers: &'lifetime [Layer], FuncName: &str) -> CompilerResult<&'lifetime Layer> {
        
        for Layer in Layers {
            if let LayerKind::Function { Name, .. } = &Layer.Kind {
                if Name == FuncName {
                    return Ok(Layer);
                }
            }
        }
        Err(CompilerError::RuntimeError(format!("No function named '{}' found", FuncName)))
    }

    // ============================================
    // Utility Functions
    // ============================================

    fn CheckType(&self, Value: &Value, Type_: &Type) -> bool {
        match (Value, Type_) {
            (Value::Int(_), Type::BitPrecise('i', _)) => true,
            (Value::Float(_), Type::BitPrecise('f', _)) => true,
            (Value::Bool(_), Type::BitPrecise('b', 1)) => true,
            (Value::Unit, Type::Unit) => true,
            _ => false,
        }
    }

    fn DefaultValue(&self, Type_: &Option<Type>) -> CompilerResult<Value> {
        match Type_ {
            Some(Type::BitPrecise('i', _)) => Ok(Value::Int(0)),
            Some(Type::BitPrecise('f', _)) => Ok(Value::Float(0.0)),
            Some(Type::BitPrecise('b', _)) => Ok(Value::Bool(false)),
            Some(Type::Unit) => Ok(Value::Unit),
            _ => Err(CompilerError::TypeError("Cannot infer default type".to_string())),
        }
    }
    /// Looks the name up in the Builtins map. Returns Some(result) if it was a
    /// built-in (handled here), or None so the caller falls through to user functions.
    fn CallBuiltin(&mut self, Name: &str, Args: &[Value]) -> Option<CompilerResult<Value>> {
        // Copy the fn pointer out of the map so the borrow on self.Builtins ends
        // before we hand &mut self to the built-in.
        let Func = self.Builtins.get(Name).copied()?;
        Some(Func(self, Args))
    }
}

// ============================================
// Built-in Function Implementations
// ============================================

/// prints its arguments separated by spaces, no trailing newline
fn Builtin_Print(_Runner: &mut CodeRunner, Args: &[Value]) -> CompilerResult<Value> {
    let Line = Args.iter().map(FormatValue).collect::<Vec<_>>().join(" ");
    print!("{}", Line);
    Ok(Value::Unit)
}

/// like print, but adds a trailing newline
fn Builtin_Println(_Runner: &mut CodeRunner, Args: &[Value]) -> CompilerResult<Value> {
    let Line = Args.iter().map(FormatValue).collect::<Vec<_>>().join(" ");
    println!("{}", Line);
    Ok(Value::Unit)
}

// ============================================
// Supporting Types
// ============================================

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Struct(Vec<(String, Value)>),
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    Variables: std::collections::HashMap<String, VariableEntry>,
    Stack: Vec<Frame>,
}

#[derive(Debug, Clone)]
pub struct VariableEntry {
    Value: Value,
    IsMutable: bool,
}

impl ExecutionContext {
    pub fn New() -> Self {
        Self {
            Variables: std::collections::HashMap::new(),
            Stack: Vec::new(),
        }
    }

    pub fn PushFrame(&mut self, Name: &str) {
        self.Stack.push(Frame {
            Name: Name.to_string(),
            Variables: std::collections::HashMap::new(),
        });
    }

    pub fn PopFrame(&mut self) {
        self.Stack.pop();
    }

    pub fn SetVariable(&mut self, Name: &str, Value: Value, IsMutable: bool) {
        if let Some(Frame) = self.Stack.last_mut() {
            Frame.Variables.insert(
                Name.to_string(),
                VariableEntry { Value, IsMutable },
            );
        } else {
            self.Variables.insert(
                Name.to_string(),
                VariableEntry { Value, IsMutable },
            );
        }
    }

    pub fn GetVariable(&self, Name: &str) -> Option<Value> {
        // Check stack frames (LIFO order)
        for Frame in self.Stack.iter().rev() {
            if let Some(Entry) = Frame.Variables.get(Name) {
                return Some(Entry.Value.clone());
            }
        }
        // Check global variables
        self.Variables.get(Name).map(|Entry| Entry.Value.clone())
    }

    pub fn Invalidate(&mut self, Target: &str) {
        // Invalidate cache for target
        // (Implementation depends on cache system)
    }

    pub fn KeepCache(&mut self, Target: &str) {
        // Keep specified value in cache
        // (Implementation depends on cache system)
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub Name: String,
    pub Variables: std::collections::HashMap<String, VariableEntry>,
}

// ============================================
// Placeholder Types (if not defined elsewhere)
// ============================================

// These should be defined in your ast crate
// Adding them here as placeholders to make the code compile

#[derive(Debug, Clone, PartialEq)]
pub enum TraceEvent {
    Havoc { Target: String },
    Read { Variable: String, Value: Value },
    Write { Variable: String, Old: Value, New: Value },
    FunctionCall { Name: String, Args: Vec<Value>, Result: Value },
    Interrupt { Syscall: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerTrace {
    Events: Vec<TraceEvent>,
    Layers: Vec<Layer>,
}

impl LayerTrace {

    pub fn NewFrom(Layers: &[Layer]) -> Self {
        Self {
            Events: Vec::new(),
            Layers: Layers.to_vec(),
        }
    }
    pub fn New() -> Self {
        Self {
            Events: Vec::new(),
            Layers: Vec::new(),
        }
    }

    pub fn RecordEvent(&mut self, Event: TraceEvent) {
        self.Events.push(Event);
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_literal_int() {
        let mut Runner = CodeRunner::New(CompilerConfig::default());
        let Expr = Expression::LiteralInt(42);
        let Result = Runner.EvaluateExpression(&Expr).unwrap();
        assert_eq!(Result, Value::Int(42));
    }

    #[test]
    fn test_evaluate_binary_add() {
        let mut Runner = CodeRunner::New(CompilerConfig::default());
        let Expr = Expression::BinaryOp {
            Op: "+".to_string(),
            Lhs: Box::new(Expression::LiteralInt(5)),
            Rhs: Box::new(Expression::LiteralInt(3)),
        };
        let Result = Runner.EvaluateExpression(&Expr).unwrap();
        assert_eq!(Result, Value::Int(8));
    }
}