# Task Plan: Unify GTK Selection, Actions, and Visible State

1. Remove index selection and add ID-based snapshot reconciliation tests.
2. Store IDs on GTK rows and route selection, activation, and keyboard actions
   through IDs.
3. Reconcile row bindings and update preview/action resolution from state.
4. Add typed command availability and collection action wiring.
5. Run targeted GTK tests, workspace tests, clippy, and `just verify`.
