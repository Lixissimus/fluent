# Fluent status Quickshell plugin

This is a first draft of the bar widget described in the project plan. It is
kept in the repository so the QML and manifest can evolve alongside Fluent.

## Expected command

The widget runs:

```sh
fluent-status --watch
```

The command is not implemented yet. The first expected response shape is:

```json
{"instances":[{"pid":1234},{"pid":5678}]}
```

The watcher should keep running and print one JSON response per line whenever
the instance list changes. The QML `SplitParser` processes each line as it
arrives. When the command exits, the widget shows an unavailable state and
restarts it after five seconds. The widget opens a small read-only window when
clicked.

## Trying it in Omarchy later

Copy or link this directory into:

```text
~/.config/omarchy/plugins/fluent.status/
```

Then rescan the shell and add `fluent.status` to the bar layout in
`~/.config/omarchy/shell.json`.

This draft uses the existing Omarchy `BarWidget`, `BarIconButton`, and
`PanelWindow` components. It should be treated as a visual/API sketch until
the `fluent-status` command and aggregator query protocol exist.