"""Example: validate a tiny crew spec (requires `bkg-peer serve --web ...`)."""

from bkg_peer import BkgPeerClient

SPEC = {
    "name": "demo",
    "agents": [
        {
            "id": "a1",
            "role": "Assistant",
            "goal": "Help",
            "backstory": "Helpful",
            "llm": "dummy",
        }
    ],
    "tasks": [{"id": "t1", "description": "Say hi", "agent_id": "a1"}],
    "process": "sequential",
}

if __name__ == "__main__":
    c = BkgPeerClient()
    print(c.validate_crew(SPEC))
