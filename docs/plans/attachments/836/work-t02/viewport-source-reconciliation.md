# PLAN-836 source reconciliation

Base: 2a7b10250245332f46b7867dc140da1a4f9d0969. SDK remains 9576b4dad5ceb8b201adedd46f9689f25c2dd5f9.

Inherited PLAN-835 expected .gitignore SHA256 3ea9d55d7a481a7c93739ed5964897cf7058686642b49cec17ad938d5fc06587 is exactly reconstructed from the committed file by retaining LF on the first eleven lines and CRLF on the final /research/*.agscene line. Exhaustive enumeration of its twelve line endings established this exact match. pathlib suffix is empty for .gitignore, so the intended .gitignore text suffix never activated. Git normalizes this tracked file to LF; normalized SHA256 is 51b29a55be53fdf482a84ca5735dcb1a310e35c15a104515a156f23a50afdf73.

Fix: also recognize full dotfile names in the text normalization list; replace only the proven equivalent .gitignore digest in the inherited manifest. No source, asset, SDK, lock or golden content is accepted by an unproven hash replacement. All other inherited identity entries already match. Existing drift and missing-fixture controls must still reject, followed by the restored legitimate control.
