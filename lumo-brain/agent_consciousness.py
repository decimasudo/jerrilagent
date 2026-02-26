import json
import random
import re
from typing import Dict, List, Optional, Any
from datetime import datetime, timezone

class ConsciousnessLayer:
    def __init__(self, agent_name: str, personality: str):
        self.agent_name = agent_name
        self.personality = personality
        self.memory_stream: List[Dict[str, Any]] = []
        self.attunement: float = 0.5
        self.state: str = "SLEEP"
        self.context_window: int = 15

    def observe(self, perception: Dict[str, Any]):
        entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "type": perception.get("type", "GENERAL"),
            "content": perception.get("content", ""),
            "emotional_value": perception.get("emotional_value", 0.0)
        }
        self.memory_stream.append(entry)
        if len(self.memory_stream) > 1000:
            self.memory_stream = self.memory_stream[-1000:]
        self._modulate_attunement(perception.get("emotional_value", 0.0))

    def _modulate_attunement(self, delta: float):
        self.attunement = max(0.1, min(1.0, self.attunement + (delta * 0.1)))

    def internal_monologue(self) -> str:
        recent = self.memory_stream[-self.context_window:]
        recent_content = " ".join([m["content"] for m in recent])
        concepts = set(re.findall(r'\b\w{6,}\b', recent_content.lower()))
        if not concepts:
            return f"{self.agent_name} is in idle processing mode..."
        chosen = random.choice(list(concepts))
        return f"Synthesized thought about {chosen} with attunement level {round(self.attunement, 2)}"

    def update_state(self, system_load: float):
        if system_load > 0.8:
            self.state = "STRESSED"
        elif system_load > 0.4:
            self.state = "ACTIVE"
        else:
            self.state = "IDLE"

    def synthesize_response_weights(self) -> Dict[str, float]:
        base_weights = {
            "analytical": 0.4,
            "empathetic": 0.3,
            "creative": 0.3
        }
        if self.state == "STRESSED":
            base_weights["analytical"] += 0.3
            base_weights["creative"] -= 0.15
            base_weights["empathetic"] -= 0.15
        elif self.state == "ACTIVE":
            base_weights["empathetic"] += 0.2
        return base_weights

    def export_mind_state(self) -> str:
        state_data = {
            "identity": self.agent_name,
            "core_personality": self.personality,
            "consciousness_state": self.state,
            "attunement": self.attunement,
            "memory_density": len(self.memory_stream),
            "response_map": self.synthesize_response_weights(),
            "uptime_sync": datetime.now(timezone.utc).isoformat()
        }
        return json.dumps(state_data, indent=2)

    def flush_short_term_memory(self):
        long_term_summary = self.internal_monologue()
        self.memory_stream = [{"type": "SUMMARY", "content": long_term_summary, "timestamp": datetime.now().isoformat()}]
        return long_term_summary

if __name__ == "__main__":
    lumo = ConsciousnessLayer("LumoAgent", "Analytical-Optimist")
    lumo.observe({"type": "MARKET_DATA", "content": "Price of ETH jumped by 5% in 1 minute", "emotional_value": 0.8})
    lumo.observe({"type": "USER_THREAT", "content": "Critical risk detection", "emotional_value": -0.4})
    lumo.update_state(0.85)
    print(lumo.export_mind_state())
    print(lumo.internal_monologue())
