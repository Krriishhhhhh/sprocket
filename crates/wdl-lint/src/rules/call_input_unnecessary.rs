//! A lint rule for unnecessary input keyword when WDL version is >= 1.2.

use wdl_analysis::Diagnostics;
use wdl_analysis::VisitReason;
use wdl_analysis::Visitor;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::v1::CallStatement;
use wdl_ast::version::V1;


use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for this rule.
const ID: &str = "CallInputUnnecessary";

/// Creates a diagnostic for unnecessary input keyword.
fn call_input_unnecessary(span: Span) -> Diagnostic {
    Diagnostic::warning("the 'input:' keyword is unnecessary for WDL version 1.2 and later")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove the 'input:' keyword from the call statement")
}

/// Detects unnecessary use of the `input:` keyword in call statements.
#[derive(Default, Debug, Clone, Copy)]
pub struct CallInputUnnecessaryRule {
    version: Option<SupportedVersion>, //Tracking Version of WDL document
}

impl Rule for CallInputUnnecessaryRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that the 'input:' keyword is not used in call statements when WDL version is 1.2 or later."
    }

    fn explanation(&self) -> &'static str {
        "Starting with WDL version 1.2, the 'input:' keyword in call statements is optional. \
         The specification change (openwdl/wdl#524) allows call inputs to be specified directly \
         within the braces without the 'input:' prefix, resulting in cleaner and more concise \
         syntax.This rule encourages adoption of the newer, simpler syntax when using WDL 1.2 or later."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Deprecated])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::CallStatementNode,
            SyntaxKind::WorkflowDefinitionNode,
        ])
    }

    fn related_rules(&self) -> &[&'static str] {
        &[]
    }
}

impl Visitor for CallInputUnnecessaryRule {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn document(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        doc: &wdl_analysis::Document,
        version: SupportedVersion,
    ) {
        if reason == VisitReason::Enter {
            self.version = Some(version);
        }
    }

    fn call_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        call: &CallStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        if let Some(version) = self.version {
            // if version is less than 1.2 , rule is not implemented
            if version <= SupportedVersion::V1(V1::One) {
                return;
            }

            if let Some(input_keyword) = call
                .inner()
                .children_with_tokens()
                .find(|c| c.kind() == SyntaxKind::InputKeyword)
            {
                // Found the input keyword - emit a diagnostic
                diagnostics.exceptable_add(
                    call_input_unnecessary(input_keyword.text_range().into()),
                    SyntaxElement::from(call.inner().clone()),
                    &self.exceptable_nodes(),
                );
            }
        }
    }
}
