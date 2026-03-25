# How to Configure Flight Levels

Flight Levels control which document types are available and how your project hierarchy is structured. Metis supports two presets that determine whether initiatives exist as an intermediate layer between visions and tasks.

## View Current Configuration

```bash
metis config show
```

```
Current Flight Level Configuration:
  Preset: streamlined
  Initiatives enabled: true

Hierarchy: Vision > Initiative > Task
          (Vision can also contain: Specification, ADR)

Available document types:
  - vision
  - initiative
  - task
  - specification
  - adr
```

## Available Presets

### Streamlined (Default)

Full hierarchy with initiatives as an intermediate planning layer.

```
Vision → Initiative → Task
```

- Tasks must belong to an initiative (or be backlog items)
- Initiatives must belong to a vision
- Best for: teams managing multiple workstreams, projects with 2+ week timelines

```bash
metis config set --preset streamlined
```

### Direct

Simplified hierarchy that skips the initiative layer.

```
Vision → Task
```

- Tasks are created directly under the vision (no initiative required)
- Best for: small projects, solo developers, quick prototypes

```bash
metis config set --preset direct
```

## Custom Configuration

Toggle initiatives independently:

```bash
metis config set --initiatives false    # Disable initiatives (same as direct)
metis config set --initiatives true     # Enable initiatives (same as streamlined)
```

## Query a Specific Setting

```bash
metis config get preset
metis config get initiatives_enabled
```

## Changing Presets Mid-Project

You can switch presets at any time. Existing documents are not deleted or moved — they remain in the filesystem. However:

- Switching from `streamlined` to `direct` means new tasks won't require an initiative parent
- Switching from `direct` to `streamlined` means new tasks will require an initiative parent
- Existing tasks retain their current parent relationships

See [Configuration Reference](../reference/configuration.md) for the config.toml format and [Flight Levels Methodology](../explanation/flight-levels.md) for the reasoning behind this hierarchy.
