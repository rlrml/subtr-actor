"""Small permutation-invariant goal models with matching scalar Rust inference."""

import numpy as np
import torch
from torch import nn

LEGACY_COUNT = 154
GLOBAL_COUNT = 160
PLAYER_COUNT = 17
FEATURE_COUNT = GLOBAL_COUNT + 4 * PLAYER_COUNT


class GoalModel(nn.Module):
    def __init__(self, mean, scale, entity=True, width=64, reflection=None):
        super().__init__()
        self.entity = entity
        self.register_buffer(
            "reflection",
            torch.as_tensor(
                np.ones(FEATURE_COUNT) if reflection is None else reflection, dtype=torch.float32
            ),
        )
        self.register_buffer("mean", torch.as_tensor(mean, dtype=torch.float32))
        self.register_buffer("scale", torch.as_tensor(scale, dtype=torch.float32))
        if entity:
            self.player1 = nn.Linear(23, 32)
            self.player2 = nn.Linear(32, 16)
            self.hidden1 = nn.Linear(GLOBAL_COUNT + 64, width)
            self.hidden2 = nn.Linear(width, 32)
            self.output = nn.Linear(32, 1)
        else:
            self.hidden1 = nn.Linear(LEGACY_COUNT, 8)
            self.output = nn.Linear(8, 1)

    def forward(self, values):
        probability = (
            self.raw_logits(values).sigmoid() + self.raw_logits(values * self.reflection).sigmoid()
        ) * 0.5
        return torch.logit(probability.clamp(1e-7, 1 - 1e-7))

    def raw_logits(self, values):
        scaled = (values - self.mean) / self.scale
        if self.entity:
            players = scaled[:, GLOBAL_COUNT:].reshape(-1, 4, PLAYER_COUNT)
            ball = scaled[:, LEGACY_COUNT:GLOBAL_COUNT].unsqueeze(1).expand(-1, 4, -1)
            players = torch.tanh(self.player1(torch.cat((players, ball), dim=2)))
            players = torch.tanh(self.player2(players)).reshape(-1, 2, 2, 16)
            pooled = torch.cat(
                (players.mean(dim=2), (players[:, :, 0] - players[:, :, 1]).abs()), dim=2
            ).flatten(1)
            hidden = torch.tanh(self.hidden1(torch.cat((scaled[:, :GLOBAL_COUNT], pooled), dim=1)))
            hidden = torch.tanh(self.hidden2(hidden))
        else:
            hidden = torch.tanh(self.hidden1(scaled[:, :LEGACY_COUNT]))
        return self.output(hidden).squeeze(1)


def normalization(values):
    mean = values.mean(axis=0, dtype=np.float64).astype(np.float32)
    scale = values.std(axis=0, dtype=np.float64).astype(np.float32)
    players = values[:, GLOBAL_COUNT:].reshape(-1, PLAYER_COUNT)
    mean[GLOBAL_COUNT:] = np.tile(players.mean(axis=0, dtype=np.float64), 4)
    scale[GLOBAL_COUNT:] = np.tile(players.std(axis=0, dtype=np.float64), 4)
    scale[scale < 1e-5] = 1.0
    return mean, scale


def labels(frame, task, horizon):
    goal_for = frame.time_to_next_goal_for.fillna(np.inf).to_numpy()
    goal_against = frame.time_to_next_goal_against.fillna(np.inf).to_numpy()
    end = frame.time_to_live_end.to_numpy()
    resolved = frame.live_end_resolved.to_numpy(dtype=bool)
    if task == "touch":
        next_touch = frame.time_to_next_touch.fillna(np.inf).to_numpy()
        boundary = np.minimum(goal_against, next_touch)
        positive = (goal_for <= horizon) & (goal_for <= end) & (goal_for < boundary)
        observed = positive | (boundary <= end) | (end >= horizon) | resolved
    else:
        positive = (goal_for <= horizon) & (goal_for <= end) & (goal_for < goal_against)
        observed = positive | (goal_against <= end) | (end >= horizon) | resolved
    return positive.astype(np.float32), observed
