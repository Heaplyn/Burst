#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast;
use ast::*;
use std::collections::*;
use std::task::Context;
use config::*;
use lexer::*;
use parser::*;
use elaboration::*;

// ============================================
// Compiler Result Type
// ============================================

pub type CompilerResult<T> = Result<T, CompilerError>;

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
#[derive(Debug, Clone)]
pub struct CodeRunner {
    Context: ElaborationContext,
    Trace: LayerTrace,
    Config: CompilerConfig,
}

impl CodeRunner {
    pub fn New(Config: CompilerConfig,Context: ElaborationContext) -> Self {
        Self {
            Context,
            Trace: LayerTrace::New(),
            Config,
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
        // Build trace from layers
        self.Trace = LayerTrace::NewFrom(Layers);
        
        // Find main function
        let MainLayer = self.FindMainFunction(Layers)?;
        
        // Execute main
        self.RunLayer(MainLayer)
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
}

impl LayerTrace {

    pub fn NewFrom(_Layers: &[Layer]) -> Self {
        Self {
            Events: Vec::new(),
        }
    }
    pub fn New() -> Self {
        Self {
            Events: Vec::new(),
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
        let mut Runner = CodeRunner::New(CompilerConfig::default(), ElaborationContext::New());
        let Expr = Expression::LiteralInt(42);
        let Result = Runner.EvaluateExpression(&Expr).unwrap();
        assert_eq!(Result, Value::Int(42));
    }

    #[test]
    fn test_evaluate_binary_add() {
        let mut Runner = CodeRunner::New(CompilerConfig::default(),ElaborationContext::New());
        let Expr = Expression::BinaryOp {
            Op: "+".to_string(),
            Lhs: Box::new(Expression::LiteralInt(5)),
            Rhs: Box::new(Expression::LiteralInt(3)),
        };
        let Result = Runner.EvaluateExpression(&Expr).unwrap();
        assert_eq!(Result, Value::Int(8));
    }
}