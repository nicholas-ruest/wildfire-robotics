# Aerial Deployment Operations schema namespaces

The owning context is `aerial-deployment-operations`. Its PostgreSQL schema is
`aerial_deployment_operations`, event subjects use
`wildfire.aerial_deployment_operations.v1.<event-name>`, and Protobuf packages
use `wildfire.aerial_deployment.v1`.

Only the owning context may publish the events listed in
`docs/architecture/aerial-deployment-ownership.toml`. Cross-context data enters
through versioned contracts or ports; aircraft/vendor, ruv-drone, persistence,
broker, web and simulation types never enter the domain namespace. Contract
evolution adds a new versioned namespace and must not reinterpret an existing
field or event.
