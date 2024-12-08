//! ES2022: Class Properties
//! Utility functions.

use std::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::{BoundIdentifier, TraverseCtx};

/// Create assignment to a binding.
pub(super) fn create_assignment<'a>(
    binding: &BoundIdentifier<'a>,
    value: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    ctx.ast.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        binding.create_target(ReferenceFlags::Write, ctx),
        value,
    )
}

/// Create `var` declaration.
pub(super) fn create_variable_declaration<'a>(
    binding: &BoundIdentifier<'a>,
    init: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    let kind = VariableDeclarationKind::Var;
    let declarator = ctx.ast.variable_declarator(
        SPAN,
        kind,
        binding.create_binding_pattern(ctx),
        Some(init),
        false,
    );
    Statement::from(ctx.ast.declaration_variable(SPAN, kind, ctx.ast.vec1(declarator), false))
}

/// Convert an iterator of `Expression`s into an iterator of `Statement::ExpressionStatement`s.
pub(super) fn exprs_into_stmts<'a, 'c, E>(
    exprs: E,
    ctx: &'c TraverseCtx<'a>,
) -> impl Iterator<Item = Statement<'a>> + 'c
where
    E: IntoIterator<Item = Expression<'a>>,
    <E as IntoIterator>::IntoIter: 'c,
{
    exprs.into_iter().map(|expr| ctx.ast.statement_expression(SPAN, expr))
}

/// Create `IdentifierName` for `_`.
pub(super) fn create_underscore_ident_name<'a>(ctx: &mut TraverseCtx<'a>) -> IdentifierName<'a> {
    ctx.ast.identifier_name(SPAN, Atom::from("_"))
}

#[inline]
pub(super) fn assert_expr_neither_parenthesis_nor_typescript_syntax(expr: &Expression) {
    debug_assert!(
        !(matches!(expr, Expression::ParenthesizedExpression(_)) || expr.is_typescript_syntax()),
        "Should not be: {expr:?}",
    );
}

/// Create array of length `N`, with each item initialized with provided function `init`.
#[inline]
pub(super) fn create_array<const N: usize, T, I: FnMut() -> T>(mut init: I) -> [T; N] {
    // https://github.com/rust-lang/rust/issues/62875#issuecomment-513834029
    // https://github.com/rust-lang/rust/issues/61956
    let mut array: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];
    for elem in &mut array {
        elem.write(init());
    }
    // Wrapping in `ManuallyDrop` should not be necessary because `MaybeUninit` does not impl `Drop`,
    // but do it anyway just to make sure, as it's mentioned in issues above.
    let mut array = ManuallyDrop::new(array);
    // SAFETY: All elements of array are initialized.
    // `[MaybeUninit<T>; N]` and `[T; N]` have equivalent layout.
    unsafe { ptr::from_mut(&mut array).cast::<[T; N]>().read() }
}
