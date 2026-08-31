//! Expression evaluation: turning an [`ast::Expression`] into a [`Value`],
//! including the function-call machinery (built-ins first, then user functions).

use ast::{Expression, Layer, LayerKind};

use parser::error::{CompilerError, CompilerResult};
use crate::runner::CodeRunner;
use crate::value::Value;

impl CodeRunner {
    pub fn EvaluateExpression(&mut self, Expr: &Expression) -> CompilerResult<Value> {
        match Expr {
            Expression::LiteralInt(Val) => Ok(Value::Int(*Val)),
            Expression::LiteralFloat(Val) => Ok(Value::Float(*Val)),
            Expression::LiteralBool(Val) => Ok(Value::Bool(*Val)),
            Expression::LiteralString(Val) => Ok(Value::String(Val.clone())),
            Expression::Variable(Name) => self
                .Context
                .GetVariable(Name)
                .ok_or_else(|| CompilerError::RuntimeError(format!("Variable '{}' not found", Name))),
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
                let FoundFunc: Layer = self.FindFunctionByName(&self.Trace.Layers, Name)?.clone();

                // 4. Push a fresh frame, bind parameters to argument values, run the body.
                self.Context.PushFrame(Name);
                if let LayerKind::Function { Params: DeclParams, .. } = &FoundFunc.Kind {
                    if DeclParams.len() != ArgValues.len() {
                        self.Context.PopFrame();
                        return Err(CompilerError::RuntimeError(format!(
                            "Function '{}' expected {} arg(s), got {}",
                            Name,
                            DeclParams.len(),
                            ArgValues.len()
                        )));
                    }
                    for (P, V) in DeclParams.iter().zip(ArgValues.into_iter()) {
                        let BaseType = match &P.Type_ {
                            ast::Type::Where(Base, _) => Base.as_ref(),
                            Other => Other,
                        };
                        //Dont need this cause we now support diff type in where or statement
                        /*if !self.CheckType(&V, BaseType) {
                            self.Context.PopFrame();
                            return Err(CompilerError::TypeError(format!(
                                "Parameter '{}' type mismatch: expected {:?}, got {:?}",
                                P.Name, BaseType, V
                            )));
                        }*/
                        self.Context.SetVariable(&P.Name, V, false);
                    }

                    // Validate refinement constraints
                    for P in DeclParams {
                        if let ast::Type::Where(_, ConstraintExpr) = &P.Type_ {
                            let Checked = self.EvaluateExpression(ConstraintExpr)?;
                            if Checked == Value::Bool(false) {
                                self.Context.PopFrame();
                                /*return Err(CompilerError::TypeError(format!(
                                    "Parameter '{}' failed refinement check in call to '{}'",
                                    P.Name, Name
                                )));*/
                                return Ok(Value::Unit);
                            }
                        }
                    }
                }
                let Result = self.RunBlock(&FoundFunc.Children)?;
                self.Context.PopFrame();
                
                if let Some(RetVal) = self.ReturnValue.take() {
                    Ok(RetVal)
                } else {
                    Ok(Result)
                }
            }
            _ => Err(CompilerError::RuntimeError("Unsupported expression".to_string())),
        }
    }

    fn EvaluateBinaryOp(&mut self, Op: &str, Lhs: Value, Rhs: Value) -> CompilerResult<Value> {
        match (Op, Lhs, Rhs) {
            // Arithmetic
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
            ("-", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Float(Lhs - Rhs)),
            ("*", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Float(Lhs * Rhs)),
            ("/", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Float(Lhs / Rhs)),

            // Comparisons
            ("<", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Bool(Lhs < Rhs)),
            ("<=", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Bool(Lhs <= Rhs)),
            (">", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Bool(Lhs > Rhs)),
            (">=", Value::Int(Lhs), Value::Int(Rhs)) => Ok(Value::Bool(Lhs >= Rhs)),

            ("<", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Bool(Lhs < Rhs)),
            ("<=", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Bool(Lhs <= Rhs)),
            (">", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Bool(Lhs > Rhs)),
            (">=", Value::Float(Lhs), Value::Float(Rhs)) => Ok(Value::Bool(Lhs >= Rhs)),

            ("==", Lhs, Rhs) => Ok(Value::Bool(Lhs == Rhs)),
            ("!=", Lhs, Rhs) => Ok(Value::Bool(Lhs != Rhs)),

            // Logic
            ("&&", Value::Bool(Lhs), Value::Bool(Rhs)) => Ok(Value::Bool(Lhs && Rhs)),
            ("||", Value::Bool(Lhs), Value::Bool(Rhs)) => Ok(Value::Bool(Lhs || Rhs)),

            _ => Err(CompilerError::RuntimeError(format!("Invalid binary operation: {}", Op))),
        }
    }
}
