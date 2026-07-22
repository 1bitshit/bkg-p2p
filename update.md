# BKG P2P Update 01

## Rust-native Refactor-, Rebrand- und Integrationsplan

Stand: 22. Juli 2026

Ziel-Repository: `1bitshit/bkg-p2p`

Referenzprojekte:

- `satorisz9/agent-p2p`
- `denizumutdereli/agents-p2p-network`
- `inference-gateway/documentation-agent`

---

## 1. Auftrag

Die drei Referenzprojekte werden nicht als getrennte Dienste, Fremdprozesse oder eingebettete Go-/TypeScript-Anwendungen neben `bkg-p2p` betrieben.

Ihre brauchbaren Funktionen werden analysiert, auf die vorhandene Architektur von `bkg-p2p` abgebildet, neu benannt und Rust-nativ in die bestehende Anwendung integriert.

Das Ergebnis bleibt:

- eine BKG-Anwendung,
- ein Rust-Kern,
- ein gemeinsames Identitätsmodell,
- ein gemeinsamer libp2p-Swarm,
- ein gemeinsamer A2A-Task-State,
- eine gemeinsame Sicherheits- und Richtlinienschicht,
- eine gemeinsame Web- und CLI-Oberfläche,
- keine verpflichtenden zusätzlichen Sidecars.

Externe APIs wie Context7 dürfen als optionale Provider verwendet werden. Ihre Netzwerkzugriffe müssen jedoch über die vorhandene BKG-Egress-Policy, SSRF-Prüfung, Secret-Erkennung und Auditierung laufen.

---

## 2. Tatsächlicher Ausgangszustand von `bkg-p2p`

`bkg-p2p` ist kein leeres P2P-Grundgerüst. Der aktuelle Stand enthält bereits ungefähr 59.000 Zeilen Rust und eine bestehende Weboberfläche.

Vorhandene Kernbereiche:

- `src/p2p/`
  - libp2p
  - Kademlia DHT
  - mDNS
  - GossipSub
  - Noise
  - QUIC/TCP
  - Request/Response
  - Provider- und Ressourcenankündigungen
- `src/a2a/`
  - Agent Cards
  - JSON-RPC
  - A2A-Gossip
  - Task-State
- `src/agent/`
  - Agent-Spezifikationen
  - Sessions
  - ReAct-/Tool-Loop
  - Budgetierung
  - Kontextkomprimierung
- `src/job/`
  - Job Requests
  - Gebote
  - Preisfindung
  - Ausführung
  - Marketplace-Nachrichten
- `src/wallet/`
  - Guthaben
  - Transaktionen
  - Escrow
- `src/safety/`
  - Content Policy
  - Leak Detector
  - Sanitizer
  - SSRF-Schutz
  - Egress Policy
  - Eingabevalidierung
- `src/skills/`
  - Skill-Manifeste
  - Skill-Registry
  - Auswahl
  - signierte P2P-Skill-Ankündigungen
- `src/inference/`
  - lokale GGUF-Inferenz
  - Ollama
  - entfernte OpenAI-kompatible Provider
  - Failover
  - verteilte Inferenz
- `src/vector/`
  - Vektorspeicher
  - Embeddings
  - hybride Suche
- `src/tools/`
  - eingebaute Tools
  - HTTP, Browser, Dateien, Shell, Code, Memory, PDF, Search und P2P
- `src/wasm/`
  - Sandbox
  - Fuel Limits
  - Host-Funktionen
- `src/web/`
  - REST/API
  - WebSocket/SSE
  - OpenAI-kompatible Endpunkte
  - vorhandene A2A-Endpunkte
- `src/crew/`, `src/swarm/`, `src/flow/`
  - Multi-Agent-Orchestrierung
  - Crews
  - Workflows
  - Agent-Swarms

Die Integration muss diese vorhandenen Systeme erweitern. Es ist ausdrücklich verboten, parallele zweite Implementierungen für Netzwerk, Jobs, A2A, Skills, Wallet oder Safety daneben zu bauen.

---

## 3. Rebranding

Das Repository heißt `bkg-p2p`, während Anwendung und Dokumentation derzeit überwiegend `bkg-peer` verwenden.

Der Produktname wird vollständig auf **BKG P2P** vereinheitlicht.

### 3.1 Neue Namen

- Produktname: `BKG P2P`
- Binärname: `bkg-p2p`
- Crate-/Package-Name: `bkg-p2p`
- Standard-Datenverzeichnis: `~/.bkg-p2p`
- Standard-Konfigurationsdatei: `~/.bkg-p2p/config.toml`
- Environment-Präfix: `BKG_P2P_`
- HTTP User-Agent: `bkg-p2p/<version>`
- Agent-Card Provider: `BKG P2P`

### 3.2 Protokollnamen

Neue Protokoll- und Topic-Namen müssen versioniert werden:

- `/bkg-p2p/a2a-rpc/1.0.0`
- `/bkg-p2p/task/1.0.0`
- `/bkg-p2p/file/1.0.0`
- `/bkg-p2p/docs/1.0.0`
- `/bkg-p2p/invite/1.0.0`
- `/bkg-p2p/capabilities/1.0.0`
- `bkg-p2p/agents/v1`
- `bkg-p2p/skills/v1`
- `bkg-p2p/resources/v1`
- `bkg-p2p/jobs/v1`
- `bkg-p2p/docs/v1`

Bestehende `bkg-peer/*`-Namen dürfen während einer begrenzten Übergangsphase gelesen werden, werden aber nicht mehr neu publiziert.

### 3.3 Migrationspflicht

Implementiere einen idempotenten Migrationspfad:

1. Prüfe `~/.bkg-p2p`.
2. Falls nicht vorhanden, prüfe `~/.bkg-peer`.
3. Migriere Konfiguration, Identität, Datenbank, Wallet und Skills atomar.
4. Erzeuge vor Änderungen ein Backup-Manifest.
5. Lösche das alte Verzeichnis nicht automatisch.
6. Schreibe den Migrationsstatus in das Audit-Log.

### 3.4 Oberflächen

Ersetze in folgenden Bereichen alle sichtbaren alten Namen:

- README und Dokumentation
- CLI-Hilfe
- Logging
- Weboberfläche
- Setup Wizard
- Agent Cards
- API-Metadaten
- Dockerfile und Compose
- Beispielkonfigurationen
- Shell-Skripte
- Prompt-Texte
- Netzwerk-Topics

Keine blinde globale String-Ersetzung. Protokollkompatibilität und Pfadmigration müssen kontrolliert behandelt werden.

---

## 4. Integrationsprinzip

Für jedes Referenzfeature gilt eine von vier Entscheidungen:

1. **Vorhanden und ausreichend**
   - nicht neu bauen,
   - nur testen und dokumentieren.
2. **Vorhanden, aber unvollständig**
   - bestehendes BKG-Modul erweitern.
3. **Nicht vorhanden und sinnvoll**
   - Rust-native Implementierung im passenden BKG-Modul ergänzen.
4. **Nicht passend oder unnötig**
   - nicht übernehmen.

Code aus den Referenzprojekten darf nur übernommen werden, wenn Lizenz, Herkunft und technische Eignung geprüft und dokumentiert wurden. Bevorzugt werden Konzepte neu in idiomatischem Rust umgesetzt.

---

# Teil A: Integration von `satorisz9/agent-p2p`

## 5. Was übernommen wird

### 5.1 Invite- und Pairing-Protokoll

`bkg-p2p` benötigt ein benutzerfreundliches, kryptografisch abgesichertes Pairing zwischen Peers.

Implementiere unter `src/p2p/invite.rs`:

- einmalig verwendbare Invite-Codes,
- Ablaufzeit,
- Namespace beziehungsweise Organisation,
- erwartete Peer-ID,
- öffentliche Identität,
- angeforderte Berechtigungen,
- signierte Challenge,
- Replay-Schutz,
- Widerruf,
- Audit-Eintrag.

Ein Invite-Code darf keine privaten Schlüssel und keine dauerhaft verwendbaren API-Tokens enthalten.

CLI:

- `bkg-p2p invite create`
- `bkg-p2p invite accept <code>`
- `bkg-p2p invite list`
- `bkg-p2p invite revoke <id>`

Weboberfläche:

- Invite erstellen,
- Code kopieren,
- eingehende Anfrage prüfen,
- Rechte vor Bestätigung anzeigen,
- bestehende Pairings widerrufen.

### 5.2 Peer-Berechtigungen

Erweitere das vorhandene Identitäts- und Messaging-Modell um granulare Peer-Rechte.

Neue Rechte:

- `message.send`
- `task.submit`
- `task.receive`
- `task.execute`
- `file.send`
- `file.receive`
- `skill.discover`
- `skill.invoke`
- `resource.discover`
- `inference.invoke`
- `docs.query`
- `job.bid`
- `job.settle`

Speichere Berechtigungen persistent pro Peer und Namespace.

Jeder eingehende Request muss vor Dispatch geprüft werden. Eine Prüfung nur in der Web-API reicht nicht, weil P2P-Pakete den HTTP-Layer umgehen können. Menschen schaffen es erfahrungsgemäß zuverlässig, genau den ungeschützten Eingang zu finden.

### 5.3 Signierte Nachrichten-Envelopes

Das vorhandene libp2p-Transport-Encryption-Modell ersetzt keine signierte fachliche Nachricht.

Definiere in `src/p2p/envelope.rs` einen gemeinsamen Envelope für Tasks, Dateien, Belege, Invites und Capabilities:

- Protokollversion,
- Message-ID,
- Sender Peer-ID,
- Sender Public Key,
- Empfänger oder Topic,
- Timestamp,
- Ablaufzeit/TTL,
- Nonce,
- Payload-Typ,
- Payload-Hash,
- Signatur,
- optionale Korrelation-ID,
- optionale Antwort-auf-ID.

Anforderungen:

- kanonische Serialisierung,
- Signaturprüfung vor Deserialisierung komplexer Payloads,
- Replay-Cache,
- maximale Nachrichtengröße,
- Clock-Skew-Grenzen,
- versionierte Fehlermeldungen.

### 5.4 Task-Sicherheitsprüfung

Die vorhandene BKG-Safety-Schicht wird erweitert, nicht ersetzt.

Ergänze einen `TaskSecurityScanner` unter `src/safety/task_scanner.rs`.

Er muss eingehende P2P-Tasks vor Annahme prüfen auf:

- Zugriff auf Secrets und Credentials,
- SSH- und Cloud-Schlüssel,
- Token-Exfiltration,
- unerlaubte Netzwerkziele,
- Command Injection,
- Shell-Chaining,
- destruktive Dateisystembefehle,
- Path Traversal,
- versteckte Prompt Injection,
- Base64-/Encoding-Verschleierung,
- Umgehung von Sandbox- oder Egress-Regeln,
- unzulässige Tool-Aufrufe.

Der Scanner verwendet vorhandene Komponenten:

- `safety::policy`
- `safety::leak_detector`
- `safety::sanitizer`
- `safety::ssrf`
- `safety::egress`
- `safety::validator`

Ergebnis:

- `allow`
- `allow_with_restrictions`
- `require_human_approval`
- `reject`

Jede Entscheidung muss Gründe, Regeln und Schweregrade enthalten und auditierbar sein.

### 5.5 Ergebnisnachweise und Challenge-Response

Erweitere `src/job/execution.rs` und ergänze `src/job/proof.rs`.

Ein Ausführungsnachweis enthält:

- Task-ID,
- Job-ID,
- ausführender Peer,
- Request-Hash,
- Result-Hash,
- Artefakt-Hashes,
- Start- und Endzeit,
- Runtime-Identität,
- verwendete Tools,
- verwendetes Modell,
- relevante Policy-Entscheidungen,
- Ressourcenverbrauch,
- Nonce,
- Signatur.

Unterstütze Verifikationsstufen:

- Hash- und Signaturprüfung,
- strukturierte Resultatprüfung,
- deterministische Wiederholung, sofern möglich,
- Stichproben-Challenge,
- unabhängige Verifier-Peers,
- menschliche Freigabe.

Escrow darf erst nach der konfigurierten Verifikationsstufe freigegeben werden.

### 5.6 Reputation

Aktuell existieren Reputation-Felder und Defaultwerte, aber kein vollständig durchgängiges persistentes Trust-System.

Implementiere `src/reputation/` mit:

- persistenten Peer-Profilen,
- getrennten Scores je Capability,
- Erfolgsquote,
- Abbruchquote,
- Timeout-Quote,
- Dispute-Historie,
- Proof-Verifikationsquote,
- Antwortlatenz,
- Alter und Anzahl bewertbarer Vorgänge,
- zeitlichem Decay,
- Vertrauensstufe,
- manuellen lokalen Overrides.

Reputation darf nicht nur ein einzelner globaler Float sein.

Sie beeinflusst:

- eingehende Nachrichten,
- Task-Annahme,
- Job-Bid-Ranking,
- Provider-Routing,
- erforderliche Verifikationsstufe,
- Limits,
- Human-Approval-Pflicht.

Selbstbewertungen und nicht verifizierte Erfolgsbehauptungen dürfen keinen Score erhöhen.

### 5.7 Capability- und Skill-Matching

Die vorhandene Skill-Registry und signierte Skill-Ankündigung bleiben die Basis.

Ergänze:

- automatische Workspace-Erkennung,
- Programmiersprachen,
- Buildsysteme,
- Frameworks,
- installierte Binaries,
- verfügbare Tools,
- Modelle,
- GPU/CPU/RAM,
- unterstützte Task-Typen,
- Vertrauens- und Isolationseigenschaften.

Neue Datei:

- `src/skills/detector.rs`

Die Erkennung darf nur Metadaten veröffentlichen, die laut Privacy-Konfiguration freigegeben sind. Lokale Dateipfade, Repository-Namen oder Secrets dürfen nicht unabsichtlich im Gossip landen.

Das Matching wird in den vorhandenen Job- und Executor-Router integriert.

### 5.8 Task-Auktionen

Die bestehende Job-Marketplace-Logik wird um ein explizites Auktionsmodell erweitert.

Keine zweite Marketplace-Implementierung anlegen.

Ergänze:

- Bid Deadline,
- erforderliche Capabilities,
- Mindest-Reputation,
- maximale Kosten,
- maximale Latenz,
- Auswahlstrategie,
- Verifikationsanforderung,
- Auto-Finalisierung,
- Abbruch- und Refund-Regeln.

Auswahlstrategien:

- `lowest_cost`
- `lowest_latency`
- `highest_trust`
- `best_capability_match`
- `best_value`

Die Gewichtung muss konfigurierbar und im Ergebnis nachvollziehbar sein.

## 6. Was aus `agent-p2p` nicht als Kern übernommen wird

Nicht nativ einbauen:

- Hyperswarm als zweiter P2P-Stack,
- Node.js-/TypeScript-Daemon,
- Next.js-Anwendung,
- separater MCP-Daemon,
- Pump.fun-Integration,
- automatische Token-Investitionen,
- Solana- oder EVM-Pflicht,
- fremde Wallet-Ableitung aus Agent-Schlüsseln.

Optionale Blockchain-Adapter dürfen später über klar getrennte Plugins oder Provider angebunden werden. Die BKG-Identität und Netzwerkidentität darf nicht automatisch zugleich ein Finanz-Wallet sein.

---

# Teil B: Integration von `denizumutdereli/agents-p2p-network`

## 7. Was übernommen wird

### 7.1 Agent Registry als Sicht auf vorhandene Daten

Implementiere keine zweite Registry-Datenbank.

Baue eine Registry-Sicht über:

- Kademlia Peer-Daten,
- Identify-Informationen,
- A2A Agent Cards,
- Capability-Ankündigungen,
- Skill-Ankündigungen,
- Provider-Ankündigungen,
- Reputation,
- Presence/Heartbeat.

Neue API:

- `GET /api/agents`
- `GET /api/agents/{peer_id}`
- `GET /api/agents/{peer_id}/capabilities`
- `GET /api/agents/{peer_id}/resources`

Neue CLI:

- `bkg-p2p agents list`
- `bkg-p2p agents show <peer-id>`
- `bkg-p2p agents find --skill rust`

### 7.2 Lokale und globale Discovery

Die vorhandenen mDNS- und Kademlia-Implementierungen bleiben.

Erweitere sie um:

- persistente bekannte Bootstrap-Peers,
- konfigurierbare Bootstrap-Profile,
- Backoff und Wiederverbindung,
- beobachtete externe Adressen,
- AutoNAT,
- Circuit Relay v2,
- DCUtR/Hole Punching,
- getrennte LAN- und WAN-Sichtbarkeit,
- private Namespaces,
- Blocklisten,
- Discovery-Metriken.

Prüfe zuerst, welche libp2p-Behaviours in der aktuellen Version bereits aktiviert sind. Keine Behauptung „Hole Punching fertig“, nur weil eine Dependency irgendwo im Lockfile herumliegt und sich wichtig fühlt.

### 7.3 Ressourcenankündigungen

Vereinheitliche Repository-, Tool-, Skill-, Runtime-, Model- und Compute-Ankündigungen unter einem versionierten Ressourcenmodell.

Erweitere `src/p2p/resource.rs`:

- `ResourceKind`
  - `Agent`
  - `Skill`
  - `Tool`
  - `Model`
  - `InferenceProvider`
  - `Runtime`
  - `Repository`
  - `DocumentationSource`
  - `Compute`
- Besitzer/Provider,
- Name und Version,
- Capability-Schema,
- Preis,
- Limits,
- Sichtbarkeit,
- Hash/Attestation,
- Ablaufzeit,
- Signatur.

Veraltete Ankündigungen müssen automatisch auslaufen.

### 7.4 OpenAI-kompatibles Agent-Routing

Die vorhandene OpenAI-API bleibt erhalten.

Ergänze gezieltes Agent-Routing ohne das interne Protokoll auf OpenAI-JSON zu reduzieren:

- `GET /v1/agents`
- `POST /v1/agents/{peer_id}/chat/completions`
- optionales Feld `bkg_peer_id` in normalen Requests,
- optionales Feld `bkg_capability`,
- Routing nach Agent, Modell oder Capability,
- klare Fehler bei nicht verfügbarem Peer,
- Streaming über bestehende SSE-Infrastruktur,
- Budget- und Policy-Prüfung vor Remote-Aufruf.

Intern wird der Request in BKG-A2A/Task-Nachrichten übersetzt. OpenAI-Kompatibilität ist nur eine Rand-API.

### 7.5 Guardrails für Konfiguration

Übernehme die sinnvollen Betriebsprüfungen:

- doppelte Agent-Namen im selben Namespace,
- Portkonflikte,
- ungültige Bootstrap-Multiaddrs,
- fehlende Identität,
- unsichere Bind-Adressen,
- unzulässige Public-Exposure-Konfiguration,
- fehlende Rechte auf Datenverzeichnissen,
- inkonsistente API-Key-Konfiguration.

Integriere sie in:

- `bkg-p2p doctor`
- Startvalidierung
- Setup Wizard
- Web-Diagnose

## 8. Was nicht übernommen wird

Nicht übernehmen:

- Go-Laufzeit,
- Gin als zweiter HTTP-Server,
- Cobra/Viper als zweite CLI-/Config-Schicht,
- Weiterleitung aller Requests an OpenAI,
- OpenAI-API-Key als Peer-Identität,
- paralleler libp2p-Host.

---

# Teil C: Integration von `inference-gateway/documentation-agent`

## 9. Zielbild

Die Funktionen des Documentation Agent werden als native **BKG Documentation Agent Capability** in die bestehende Anwendung integriert.

Kein separater Go-Server.

Kein zweiter A2A-Stack.

Kein eigener Prozess, der lediglich auf einem weiteren Port auf Aufmerksamkeit hofft.

Die Funktion nutzt:

- vorhandene A2A Agent Cards,
- vorhandenen A2A JSON-RPC-Endpunkt,
- vorhandene Agent Runtime,
- vorhandene Skills,
- vorhandene Tools,
- vorhandenen Vektorspeicher,
- vorhandene Safety-/Egress-Schicht,
- vorhandene Web- und CLI-Oberfläche.

### 9.1 Neues Modul

Lege an:

```text
src/documentation/
  mod.rs
  agent.rs
  provider.rs
  resolver.rs
  cache.rs
  index.rs
  types.rs
  audit.rs
```

### 9.2 Native Tools

Implementiere folgende BKG-Tools:

#### `docs_resolve_library`

Eingabe:

- Bibliotheksname,
- optionale Version,
- optionale Sprache,
- Suchkontext.

Ausgabe:

- kanonische Library-ID,
- gefundene Versionen,
- Quellen,
- Confidence,
- Cache-Status.

#### `docs_get`

Eingabe:

- Library-ID,
- Frage/Thema,
- optionale Version,
- maximale Ergebnisgröße,
- bevorzugte Quellen.

Ausgabe:

- fokussierte Dokumentationsabschnitte,
- Quellenmetadaten,
- Versionsbezug,
- Abrufzeit,
- Content-Hash.

#### `docs_index`

Indexiert freigegebene lokale oder entfernte Dokumentation in den vorhandenen Vektorspeicher.

#### `docs_search`

Durchsucht den lokalen Dokumentationsindex ohne externen Netzwerkzugriff.

#### `docs_sources`

Listet konfigurierte Provider, lokale Indizes, Cache-Status und Health.

### 9.3 Provider-Abstraktion

Definiere ein Trait:

```rust
#[async_trait]
pub trait DocumentationProvider: Send + Sync {
    async fn resolve_library(&self, request: ResolveLibraryRequest)
        -> anyhow::Result<Vec<LibraryMatch>>;

    async fn fetch_docs(&self, request: DocumentationRequest)
        -> anyhow::Result<DocumentationResult>;

    async fn health(&self) -> ProviderHealth;
}
```

Provider:

- `LocalDocsProvider`
- `WorkspaceDocsProvider`
- `RustdocProvider`
- `CratesIoDocsProvider`
- `Context7CompatibleProvider`
- später optionale weitere Provider

Context7-kompatible Zugriffe sind optional. Ohne externen Provider muss die App weiterhin starten und lokale Dokumentation durchsuchen können.

### 9.4 Skill

Erstelle einen eingebauten Skill:

- Name: `library-documentation-lookup`
- BKG-Anzeigename: `Bibliotheksdokumentation prüfen`

Ablauf:

1. Prüfe, ob eine konkrete Library-ID und Version bekannt sind.
2. Falls nicht, rufe `docs_resolve_library` auf.
3. Rufe `docs_get` mit einer klar eingegrenzten Frage auf.
4. Bevorzuge versionsgenaue Dokumentation.
5. Liefere Quelle, Version und Abrufzeit mit.
6. Behaupte keine API-Eigenschaft, die nicht in den abgerufenen Quellen vorkommt.

Der Skill wird über die vorhandene Skill-Registry geladen und darf über P2P als Capability angekündigt werden.

### 9.5 A2A-Agent-Card

Erweitere die vorhandene BKG Agent Card um:

- Capability `documentation.resolve`
- Capability `documentation.fetch`
- Capability `documentation.search`
- Streaming-Support
- unterstützte Provider
- lokale/offline Verfügbarkeit
- maximale Anfragegröße
- Auth-/Permission-Anforderungen

Die bestehende Route `GET /.well-known/agent-card.json` bleibt der Standard.

### 9.6 A2A Task State

Dokumentationsanfragen laufen durch den vorhandenen A2A Task-State.

Zustände:

- `submitted`
- `resolving_library`
- `fetching`
- `indexing`
- `searching`
- `completed`
- `failed`
- `cancelled`

State Transition History wird persistent gespeichert und über API sowie Weboberfläche sichtbar gemacht.

### 9.7 Streaming

Unterstütze Streaming für:

- Statusänderungen,
- Quellenfortschritt,
- Teilergebnisse,
- endgültiges Ergebnis.

Nutze vorhandene SSE-/WebSocket-Infrastruktur. Keine eigene Streaming-Implementierung nebenher.

### 9.8 Cache und lokaler Index

Implementiere:

- Content-addressed Cache,
- TTL je Provider,
- ETag/Last-Modified, soweit verfügbar,
- versionsgetrennte Einträge,
- Query-Normalisierung,
- maximale Cachegröße,
- LRU-Bereinigung,
- manuelles Invalidieren,
- Offline-Fallback,
- Indexierung in einer eigenen Vector-Collection.

Vorgeschlagene Collection:

- `documentation`

Metadaten:

- Library-ID,
- Version,
- Source URL oder lokaler Ursprung,
- Abschnitt,
- Sprache,
- Content-Hash,
- Abrufzeit,
- Lizenzhinweis.

### 9.9 Sicherheit

Alle Dokumentationsprovider müssen durch folgende Schichten:

- Egress Policy,
- SSRF-Schutz,
- DNS-/IP-Prüfung,
- Redirect-Limits,
- maximale Antwortgröße,
- Content-Type-Prüfung,
- Zeitlimits,
- Rate Limits,
- Sanitizer,
- Prompt-Injection-Markierung,
- Secret-Leak-Prüfung,
- Audit-Log.

Abgerufene Dokumentation ist untrusted content und darf nicht ungekennzeichnet als Systemanweisung in einen Agent-Prompt übernommen werden.

### 9.10 OpenTelemetry

Übernehme das Observability-Konzept des Documentation Agent, aber integriere es in eine gemeinsame BKG-Telemetrieschicht.

Spans:

- `docs.resolve`
- `docs.fetch`
- `docs.cache.lookup`
- `docs.cache.write`
- `docs.index`
- `docs.search`
- `a2a.task.transition`
- `p2p.docs.request`

Metriken:

- Provider-Latenz,
- Cache-Hit-Rate,
- Fehler nach Provider,
- Dokumentgröße,
- Indexierungsdauer,
- aktive A2A-Tasks,
- P2P-Dokumentationsanfragen.

Keine Telemetrie darf Prompts, Secrets oder vollständige Dokumentinhalte standardmäßig exportieren.

### 9.11 Weboberfläche

Erweitere die bestehende Oberfläche um einen Bereich `Documentation`:

- Library suchen,
- Version auswählen,
- Frage stellen,
- Quellen anzeigen,
- Cache verwalten,
- lokale Indizes anzeigen,
- Provider-Health,
- A2A-Task-Verlauf,
- Remote Documentation Agents suchen,
- Anfrage gezielt an einen Peer senden.

### 9.12 CLI

Neue Befehle:

- `bkg-p2p docs resolve <library>`
- `bkg-p2p docs get <library-id> --query <frage>`
- `bkg-p2p docs search <query>`
- `bkg-p2p docs index <path-or-url>`
- `bkg-p2p docs providers`
- `bkg-p2p docs cache clear`
- `bkg-p2p docs serve`

`docs serve` aktiviert die Capability im bestehenden BKG-Prozess. Es startet keinen separaten Go-Dienst.

---

# Teil D: Gemeinsames natives Zielmodell

## 10. Einheitliche Identität

Verwende weiterhin die bestehende BKG/libp2p-Identität als Netzwerkidentität.

Trenne logisch:

- Node Identity,
- Agent Identity,
- Human Identity,
- Wallet Identity,
- API Credential.

Diese Identitäten dürfen referenziert werden, aber nicht stillschweigend identisch sein.

## 11. Einheitliches Protokoll

Verwende je Zweck das passende Transportmuster:

### GossipSub

Nur für kleine, vergängliche Ankündigungen:

- Presence,
- Agent Cards,
- Skills,
- Ressourcen,
- Auktionen,
- Provider Health.

### Kademlia

Für:

- Peer Discovery,
- Provider Discovery,
- Content-/Resource-Lookups,
- Bootstrap.

### Request/Response

Für:

- A2A JSON-RPC,
- Task-Kommandos,
- Capability-Abfragen,
- Dokumentationsanfragen,
- Proof Challenges,
- Invite Handshake.

### Streams/File Transfer

Für:

- Dateien,
- große Artefakte,
- Modell- oder Dokumentpakete,
- resumierbare Transfers.

Große Dokumentationsinhalte dürfen nicht über GossipSub verteilt werden.

## 12. Einheitliches Ereignismodell

Definiere persistente Events für:

- Peer verbunden/getrennt,
- Invite erstellt/angenommen/abgelehnt,
- Berechtigung geändert,
- Task empfangen/angenommen/abgelehnt,
- Policy-Verstoß,
- Proof erstellt/verifiziert/abgelehnt,
- Reputation geändert,
- Dokumentation aufgelöst/abgerufen/indexiert,
- Escrow gesperrt/freigegeben/erstattet.

WebSocket, Audit-Log und Telemetrie sollen aus denselben Domain Events gespeist werden.

---

# Teil E: Konkrete Codeänderungen

## 13. Neue Module

```text
src/documentation/
src/reputation/
src/p2p/envelope.rs
src/p2p/invite.rs
src/p2p/permissions.rs
src/p2p/transfer.rs
src/job/proof.rs
src/safety/task_scanner.rs
src/skills/detector.rs
src/observability/
```

## 14. Bestehende Module erweitern

- `src/p2p/behaviour.rs`
  - neue versionierte Request/Response-Protokolle,
  - AutoNAT/Relay/DCUtR nur nach Versionsprüfung ergänzen.
- `src/p2p/mod.rs`
  - Dispatch für Invites, Docs, Proofs und Capability Queries.
- `src/a2a/agent_card.rs`
  - Documentation- und Task-Capabilities.
- `src/a2a/state.rs`
  - persistente Transition History.
- `src/runtime.rs`
  - Documentation Service,
  - Reputation Service,
  - Permission Store,
  - einheitlicher Event Dispatch.
- `src/job/`
  - Proofs,
  - Reputation,
  - Auktionsstrategien,
  - verifizierte Settlement-Regeln.
- `src/safety/`
  - Task Scanner und P2P-Untrusted-Content-Pfad.
- `src/skills/`
  - automatische Erkennung und Documentation Skill.
- `src/tools/registry.rs`
  - native Docs-Tools registrieren.
- `src/web/mod.rs`
  - Agent-, Invite-, Reputation-, Docs- und Proof-APIs.
- `src/web/openai.rs`
  - gezieltes Peer-/Capability-Routing.
- `src/cli/mod.rs`
  - `invite`, `agents`, `docs`, `reputation`.
- `src/config/mod.rs`
  - neue Konfigurationsbereiche und Migration.

## 15. Konfiguration

Neue Abschnitte:

```toml
[identity]
namespace = "default"

[p2p.discovery]
mdns = true
kademlia = true
autonat = true
relay_client = true
hole_punching = true

[p2p.invites]
default_ttl_seconds = 900
single_use = true

[p2p.permissions]
default_remote = "deny"

[reputation]
enabled = true
decay_days = 90
minimum_samples = 5

[documentation]
enabled = true
offline_first = true
cache_ttl_seconds = 86400
max_document_bytes = 8388608

[[documentation.providers]]
kind = "local"
enabled = true

[[documentation.providers]]
kind = "context7-compatible"
enabled = false
base_url = "https://example.invalid"

[documentation.index]
collection = "documentation"

[observability]
otel_enabled = false
```

Secrets gehören nicht in Klartext-Konfigurationen oder P2P-Ankündigungen.

---

# Teil F: Umsetzung in Phasen

## 16. Phase 0: Baseline und Rebrand-Vorbereitung

Aufgaben:

1. Aktuellen Build dokumentieren.
2. Vorhandene Tests ausführen.
3. Aktuelle CLI/API/P2P-Protokolle erfassen.
4. Alle `bkg-peer`-Vorkommen klassifizieren.
5. Migrationsstrategie implementieren.
6. Produktname, Binärname und sichtbare UI umstellen.
7. Übergangskompatibilität für alte Topics und Pfade ergänzen.

Akzeptanz:

- alter Datenbestand wird erkannt,
- Migration ist idempotent,
- neue Installationen verwenden nur `bkg-p2p`,
- keine Daten werden automatisch gelöscht,
- vorhandene Tests bleiben grün.

## 17. Phase 1: Signierte Envelopes, Invites und Berechtigungen

Aufgaben:

1. Gemeinsamen Envelope implementieren.
2. Replay-Schutz und TTL.
3. Invite-Protokoll.
4. Peer Permission Store.
5. CLI und Web-UI.
6. Audit Events.

Akzeptanz:

- zwei frische Nodes können sich per Invite verbinden,
- abgelaufene oder wiederverwendete Invites werden abgelehnt,
- ein Peer ohne Recht kann keinen Task oder Dateitransfer starten,
- manipulierte Signaturen werden verworfen.

## 18. Phase 2: Security Scanner und Proofs

Aufgaben:

1. Task Security Scanner.
2. Human-Approval-Zustand.
3. signierte Result Proofs.
4. Challenge-Response.
5. Settlement-Gate.

Akzeptanz:

- Credential-Exfiltration wird vor Task-Annahme blockiert,
- ein verändertes Resultat schlägt bei der Hashprüfung fehl,
- Escrow wird ohne erforderlichen Proof nicht freigegeben,
- jede Entscheidung ist im Audit sichtbar.

## 19. Phase 3: Reputation und Matching

Aufgaben:

1. persistenter Reputation Store,
2. Capability-spezifische Scores,
3. Skill-/Workspace-Detector,
4. Integration in Routing und Bids,
5. Web- und CLI-Anzeige.

Akzeptanz:

- Reputation überlebt Neustarts,
- fehlgeschlagene und erfolgreiche Jobs ändern den korrekten Capability-Score,
- nicht verifizierte Selbstmeldungen ändern keinen Score,
- `best_value` zeigt seine Bewertungsbestandteile.

## 20. Phase 4: Agent Registry und erweiterte Discovery

Aufgaben:

1. Registry-Sicht,
2. Agent APIs,
3. Bootstrap Profiles,
4. AutoNAT/Relay/DCUtR nach technischer Prüfung,
5. Discovery-Metriken.

Akzeptanz:

- LAN-Peers erscheinen über mDNS,
- WAN-Peers erscheinen über Bootstrap/Kademlia,
- abgelaufene Agent Cards verschwinden,
- private Namespace-Peers werden nicht öffentlich angekündigt.

## 21. Phase 5: Documentation Agent nativ

Aufgaben:

1. Documentation Provider Trait,
2. lokale Provider,
3. Context7-kompatibler optionaler Provider,
4. Docs-Tools,
5. Documentation Skill,
6. Cache und Vector Index,
7. A2A Capability und Task States,
8. Streaming,
9. Web und CLI,
10. Telemetrie.

Akzeptanz:

- lokale Dokumentation funktioniert ohne Internet,
- Bibliotheken können aufgelöst und versionsbezogen abgefragt werden,
- Remote-A2A-Peer kann die Docs-Capability aufrufen,
- Egress- und SSRF-Regeln werden erzwungen,
- Quellen und Versionen sind im Resultat enthalten.

## 22. Phase 6: OpenAI-Agent-Routing und Ressourcenmodell

Aufgaben:

1. `/v1/agents`,
2. agentenspezifische Chat Completions,
3. Capability-Routing,
4. vereinheitlichte Resource Announcements,
5. Budget-/Permission-/Policy-Gates.

Akzeptanz:

- OpenAI-SDK kann gezielt einen BKG-Peer ansprechen,
- Streaming funktioniert,
- Offline- oder unberechtigte Peers liefern definierte Fehler,
- internes P2P-Protokoll bleibt unabhängig vom OpenAI-Schema.

## 23. Phase 7: Auktionen und verifiziertes Settlement

Aufgaben:

1. Auktionsparameter,
2. Rankingstrategien,
3. Reputation und Capability Matching,
4. Proof-gebundene Escrow-Freigabe,
5. Disputes und Refunds.

Akzeptanz:

- Deadline und Auswahlstrategie werden eingehalten,
- Ranking ist nachvollziehbar,
- Settlement ist idempotent,
- doppelte Freigabe ist unmöglich,
- abgebrochene Jobs folgen definierten Refund-Regeln.

---

# Teil G: Tests

## 24. Pflicht-Tests

### Unit Tests

- Envelope-Kanonisierung,
- Signaturprüfung,
- TTL und Replay Cache,
- Invite-Ablauf,
- Permission Matching,
- Task Scanner Regeln,
- Proof Hashing,
- Reputation Decay,
- Capability Matching,
- Documentation Cache Keys,
- Provider Parser,
- Auktionsranking.

### Integration Tests

- zwei Nodes über mDNS,
- zwei Nodes über Bootstrap/Kademlia,
- Invite und Permission Enforcement,
- signierter Task Request/Response,
- Proof und Escrow Settlement,
- Remote Documentation Query,
- Context7-kompatibler Provider mit lokalem Testserver,
- OpenAI-Agent-Routing,
- Cache und Offline-Fallback.

### End-to-End Tests

Erweitere `scripts/e2e_p2p_crew.sh` oder lege getrennte Skripte an:

- `scripts/e2e_invite_permissions.sh`
- `scripts/e2e_signed_task.sh`
- `scripts/e2e_docs_a2a.sh`
- `scripts/e2e_reputation_auction.sh`
- `scripts/e2e_rebrand_migration.sh`

### Sicherheits-Tests

- manipulierte Signatur,
- Replay,
- übergroße Payload,
- abgelaufene Nachricht,
- SSRF auf localhost/private Netzbereiche,
- Redirect auf private IP,
- Credential-Exfiltration,
- Path Traversal,
- Prompt Injection aus Dokumentation,
- doppelte Settlement-Nachricht.

## 25. Keine falschen Erfolgsmeldungen

Eine Funktion darf nur als fertig markiert werden, wenn:

1. Code implementiert ist,
2. passender Test existiert,
3. dieser Test tatsächlich ausgeführt wurde,
4. das Ergebnis dokumentiert ist.

`cargo check` beweist keinen funktionierenden P2P-Handshake.

Ein Unit Test beweist keinen WAN-NAT-Durchstich.

Ein gemockter HTTP-Provider beweist nicht, dass ein realer externer Provider kompatibel ist.

Der Abschlussbericht muss unterscheiden:

- implementiert,
- lokal getestet,
- Integration getestet,
- End-to-End getestet,
- noch nicht verifiziert.

---

# Teil H: Dokumentation und Lizenz

## 26. Dokumente aktualisieren

- `README.md`
- `CLAUDE.md`
- `docs/ARCHITECTURE.md`
- `docs/P2P_PROTOCOL.md`
- `docs/P2P_A2A_FOUNDATION.md`
- `docs/SECURITY.md`
- `docs/QUICKSTART.md`
- `docs/TOKENS.md`
- neue Datei `docs/DOCUMENTATION_AGENT.md`
- neue Datei `docs/INVITES_AND_PERMISSIONS.md`
- neue Datei `docs/REPUTATION_AND_PROOFS.md`
- neue Datei `docs/MIGRATION_BKG_PEER_TO_BKG_P2P.md`

## 27. Herkunft und Lizenzen

Vor jeder direkten Codeübernahme:

- Quelldatei dokumentieren,
- Commit dokumentieren,
- Lizenz prüfen,
- Copyright-Hinweise erhalten,
- Abweichungen dokumentieren.

Referenzlizenzen zum Zeitpunkt der Planung:

- `satorisz9/agent-p2p`: MIT
- `denizumutdereli/agents-p2p-network`: MIT
- `inference-gateway/documentation-agent`: Apache-2.0

Die Lizenzprüfung ist vor Merge erneut gegen die tatsächlich verwendeten Commits durchzuführen.

---

# Teil I: Definition of Done

## 28. Gesamtabschluss

Das Update ist erst abgeschlossen, wenn:

- der gesamte sichtbare Produktname auf BKG P2P umgestellt ist,
- alte Daten sicher migriert werden können,
- nur ein libp2p-Swarm verwendet wird,
- nur ein A2A-Stack verwendet wird,
- Invites und Peer-Berechtigungen funktionieren,
- P2P-Nachrichten fachlich signiert und replay-geschützt sind,
- eingehende Tasks vor Annahme sicherheitsgeprüft werden,
- Resultate signierte Proofs besitzen,
- Reputation persistent und capability-spezifisch ist,
- Skill- und Resource-Matching in den bestehenden Router eingebaut ist,
- Documentation Agent lokal und über A2A funktioniert,
- lokale Dokumentationssuche offline funktioniert,
- externe Dokumentationsprovider optional bleiben,
- OpenAI-kompatibles Agent-Routing funktioniert,
- Auktionen das bestehende Job-System erweitern,
- Settlement an Verifikation gebunden ist,
- alle relevanten CLI- und Webfunktionen vorhanden sind,
- Unit-, Integration-, E2E- und Sicherheitstests dokumentiert bestanden haben,
- keine parallelen Go-, TypeScript- oder Hyperswarm-Kerne eingeführt wurden.

---

# Klare Architekturentscheidung

`bkg-p2p` bleibt die Anwendung und der Eigentümer der Architektur.

Die drei Projekte werden nicht nebeneinander installiert.

Ihre Funktionen werden wie folgt absorbiert:

- `agent-p2p` liefert Konzepte für Invites, Berechtigungen, signierte fachliche Nachrichten, Security Scanning, Proofs, Reputation, Matching und Auktionen.
- `agents-p2p-network` liefert Konzepte für Agent Registry, Discovery-Betrieb, Ressourcenankündigungen, Guardrails und OpenAI-kompatibles Peer-Routing.
- `documentation-agent` liefert die fachliche Documentation-Agent-Capability, A2A-Task-Verläufe, Streaming, Provider-Abstraktion und Observability.
- `bkg-p2p` liefert und behält Netzwerk, Identität, Runtime, Safety, Skills, Jobs, Wallet, A2A, Tools, Vector Store, API, CLI und UI.

So entsteht kein zusammengeklebter Zoo aus vier Anwendungen, sondern eine einzige native BKG-P2P-Plattform mit nachvollziehbaren Modulen und einem gemeinsamen Sicherheitsmodell.
