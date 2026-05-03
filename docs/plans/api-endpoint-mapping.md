# PubSub Emulator REST API → PSE Web Routes Mapping

This maps the Google PubSub v1 REST API endpoints (used by the emulator) to the PSE web UI's internal routes.

## Emulator API Endpoints (called server-side by reqwest)

| Operation              | Method   | Emulator URL                                             |
|------------------------|----------|----------------------------------------------------------|
| Create Topic           | `PUT`    | `/v1/projects/{project}/topics/{topic}`                  |
| List Topics            | `GET`    | `/v1/projects/{project}/topics`                          |
| Delete Topic           | `DELETE` | `/v1/projects/{project}/topics/{topic}`                  |
| Create Subscription    | `PUT`    | `/v1/projects/{project}/subscriptions/{sub}`             |
| List Subs on Topic     | `GET`    | `/v1/projects/{project}/topics/{topic}/subscriptions`    |
| Get Subscription       | `GET`    | `/v1/projects/{project}/subscriptions/{sub}`             |
| Delete Subscription    | `DELETE` | `/v1/projects/{project}/subscriptions/{sub}`             |
| Publish Messages       | `POST`   | `/v1/projects/{project}/topics/{topic}:publish`          |
| Pull Messages          | `POST`   | `/v1/projects/{project}/subscriptions/{sub}:pull`        |
| Acknowledge Messages   | `POST`   | `/v1/projects/{project}/subscriptions/{sub}:acknowledge` |

## PSE Web UI Routes (served by axum, consumed by HTMX)

| Route                                    | Method   | Returns          | Purpose                              |
|------------------------------------------|----------|------------------|--------------------------------------|
| `/`                                      | `GET`    | Full page        | Project selector / home              |
| `/project?project=X`                     | `GET`    | Full page        | Project workspace                    |
| `/projects`                              | `POST`   | HTML fragment    | Add project, return project card     |
| `/projects/{project}/topics`             | `GET`    | HTML fragment    | Topic list partial                   |
| `/projects/{project}/topics`             | `POST`   | HTML fragment    | Create topic, return updated list    |
| `/topics/{fqn}/detail`                   | `GET`    | HTML fragment    | Topic detail + publish form          |
| `/topics/{fqn}/delete`                   | `DELETE` | HTML fragment    | Delete topic, return updated list    |
| `/topics/{fqn}/subscriptions`            | `GET`    | HTML fragment    | Subscription list partial            |
| `/topics/{fqn}/subscriptions`            | `POST`   | HTML fragment    | Create subscription                  |
| `/subscriptions/{fqn}/detail`            | `GET`    | HTML fragment    | Subscription detail + pull UI        |
| `/subscriptions/{fqn}/delete`            | `DELETE` | HTML fragment    | Delete subscription                  |
| `/topics/{fqn}/publish`                  | `POST`   | HTML fragment    | Publish message, return toast        |
| `/subscriptions/{fqn}/pull`              | `POST`   | HTML fragment    | Pull messages, return message cards  |
| `/subscriptions/{fqn}/ack`               | `POST`   | Empty (swap del) | Ack message, HTMX removes card      |
| `/settings`                              | `GET`    | HTML fragment    | Settings modal                       |
| `/settings`                              | `PUT`    | HTML fragment    | Update settings, return toast        |
| `/static/{path}`                         | `GET`    | Static file      | HTMX js, CSS                         |

## Key Differences from Angular UI

1. **No CORS** — Browser talks only to PSE server; PSE calls emulator via gcloud-pubsub gRPC (same client as CLI)
2. **No client-side state** — No localStorage; project list from confy config
3. **No JSON API** — All routes return HTML fragments for HTMX swap
4. **Shared service layer** — Same business logic as CLI, different presentation
5. **Config-driven** — Projects and emulator hosts from existing confy config
