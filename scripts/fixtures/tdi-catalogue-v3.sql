-- Frozen deployed v2 table/index layout, before search support. Migration oracle.
PRAGMA user_version=2;
CREATE TABLE campaigns (id TEXT PRIMARY KEY, spec TEXT NOT NULL, endpoint TEXT NOT NULL, phase TEXT NOT NULL, workflow TEXT UNIQUE, admission TEXT, snapshot TEXT, created_ns INTEGER NOT NULL);
CREATE TABLE events (sequence INTEGER PRIMARY KEY, campaign TEXT NOT NULL REFERENCES campaigns(id), kind TEXT NOT NULL, payload TEXT NOT NULL, recorded_ns INTEGER NOT NULL);
CREATE INDEX campaign_events ON events(campaign, sequence);
CREATE TABLE results (campaign TEXT NOT NULL REFERENCES campaigns(id), step TEXT NOT NULL, output TEXT NOT NULL, evidence TEXT NOT NULL, PRIMARY KEY(campaign,step,output));
CREATE TABLE cache (key TEXT PRIMARY KEY, entry TEXT NOT NULL, campaign TEXT NOT NULL REFERENCES campaigns(id), step TEXT NOT NULL, output TEXT NOT NULL);
CREATE TABLE restored_artifacts (campaign TEXT NOT NULL REFERENCES campaigns(id), artifact TEXT NOT NULL, location TEXT NOT NULL, PRIMARY KEY(campaign,artifact));
CREATE TABLE exports (id TEXT PRIMARY KEY, campaign TEXT NOT NULL REFERENCES campaigns(id), kind TEXT NOT NULL, endpoint TEXT NOT NULL, plan TEXT NOT NULL, state TEXT NOT NULL, receipt TEXT NOT NULL, updated_ns INTEGER NOT NULL);
CREATE INDEX campaign_exports ON exports(campaign, updated_ns);

-- Synthetic schema-3 fixture matching PR #385; no real campaign is represented.
CREATE TABLE searches (id TEXT PRIMARY KEY, spec TEXT NOT NULL, binding TEXT NOT NULL, response TEXT NOT NULL, phase TEXT NOT NULL, sequence INTEGER NOT NULL, created_ns INTEGER NOT NULL);
CREATE TABLE search_events (sequence INTEGER PRIMARY KEY, search TEXT NOT NULL REFERENCES searches(id), kind TEXT NOT NULL, payload TEXT NOT NULL, recorded_ns INTEGER NOT NULL);
CREATE INDEX search_events_by_search ON search_events(search,sequence);
CREATE TABLE search_stages (search TEXT NOT NULL REFERENCES searches(id), attempt TEXT NOT NULL, campaign TEXT NOT NULL REFERENCES campaigns(id), roots TEXT NOT NULL, PRIMARY KEY(search,attempt));
PRAGMA user_version=3;
