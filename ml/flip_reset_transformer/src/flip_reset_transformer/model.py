from __future__ import annotations

from dataclasses import dataclass

import torch
from torch import nn


@dataclass(frozen=True)
class FlipResetTransformerConfig:
    input_dim: int
    window_length: int
    model_dim: int = 128
    encoder_type: str = "transformer"
    time_feature_dim: int = 9
    cnn_layers: int = 2
    cnn_kernel_size: int = 5
    encoder_layers: int = 4
    attention_heads: int = 4
    feedforward_dim: int = 256
    dropout: float = 0.1
    additive_position_embedding: bool = True


class TemporalConvFrontEnd(nn.Module):
    def __init__(self, model_dim: int, layers: int, kernel_size: int, dropout: float) -> None:
        super().__init__()
        if layers <= 0:
            self.layers = nn.ModuleList()
            return

        padding = kernel_size // 2
        self.layers = nn.ModuleList(
            [
                nn.Sequential(
                    nn.Conv1d(model_dim, model_dim, kernel_size=kernel_size, padding=padding),
                    nn.GELU(),
                    nn.Dropout(dropout),
                )
                for _ in range(layers)
            ]
        )

    def forward(self, hidden: torch.Tensor) -> torch.Tensor:
        if not self.layers:
            return hidden

        conv_hidden = hidden.transpose(1, 2)
        for layer in self.layers:
            residual = conv_hidden
            conv_hidden = layer(conv_hidden) + residual
        return conv_hidden.transpose(1, 2)


class TemporalConvSequenceEncoder(nn.Module):
    def __init__(self, model_dim: int, layers: int, kernel_size: int, dropout: float) -> None:
        super().__init__()
        if layers <= 0:
            self.layers = nn.ModuleList()
            return

        self.layers = nn.ModuleList(
            [
                nn.Sequential(
                    nn.Conv1d(
                        model_dim,
                        model_dim,
                        kernel_size=kernel_size,
                        dilation=2**layer_index,
                        padding=(kernel_size // 2) * (2**layer_index),
                    ),
                    nn.GELU(),
                    nn.Dropout(dropout),
                )
                for layer_index in range(layers)
            ]
        )

    def forward(self, hidden: torch.Tensor) -> torch.Tensor:
        if not self.layers:
            return hidden

        conv_hidden = hidden.transpose(1, 2)
        for layer in self.layers:
            residual = conv_hidden
            conv_hidden = layer(conv_hidden) + residual
        return conv_hidden.transpose(1, 2)


class FlipResetEncoder(nn.Module):
    def __init__(self, config: FlipResetTransformerConfig) -> None:
        super().__init__()
        self.config = config
        if config.time_feature_dim < 1 or config.time_feature_dim > 9:
            raise ValueError("time_feature_dim must be between 1 and 9")
        if config.encoder_type not in {"transformer", "tcn"}:
            raise ValueError("encoder_type must be 'transformer' or 'tcn'")
        self.input_projection = nn.Linear(config.input_dim + config.time_feature_dim, config.model_dim)
        self.temporal_frontend = TemporalConvFrontEnd(
            model_dim=config.model_dim,
            layers=config.cnn_layers,
            kernel_size=config.cnn_kernel_size,
            dropout=config.dropout,
        )
        self.position_embedding = None
        if config.encoder_type == "transformer":
            self.position_embedding = (
                nn.Embedding(config.window_length + 1, config.model_dim)
                if config.additive_position_embedding
                else None
            )
            encoder_layer = nn.TransformerEncoderLayer(
                d_model=config.model_dim,
                nhead=config.attention_heads,
                dim_feedforward=config.feedforward_dim,
                dropout=config.dropout,
                batch_first=True,
                activation="gelu",
                norm_first=True,
            )
            self.sequence_encoder: nn.Module = nn.TransformerEncoder(
                encoder_layer, num_layers=config.encoder_layers
            )
        else:
            self.sequence_encoder = TemporalConvSequenceEncoder(
                model_dim=config.model_dim,
                layers=config.encoder_layers,
                kernel_size=config.cnn_kernel_size,
                dropout=config.dropout,
            )
        if self.position_embedding is not None:
            position_indices = torch.arange(config.window_length + 1, dtype=torch.long)
            self.register_buffer("position_indices", position_indices, persistent=False)

    def build_time_features(self, relative_times: torch.Tensor) -> torch.Tensor:
        max_abs_time = relative_times.abs().amax(dim=1, keepdim=True).clamp_min(1e-6)
        normalized_time = relative_times / max_abs_time
        squared_time = normalized_time.square()
        cubic_time = normalized_time * squared_time
        pi = torch.pi
        sin_1 = torch.sin(pi * normalized_time)
        cos_1 = torch.cos(pi * normalized_time)
        sin_2 = torch.sin(2.0 * pi * normalized_time)
        cos_2 = torch.cos(2.0 * pi * normalized_time)
        sin_4 = torch.sin(4.0 * pi * normalized_time)
        cos_4 = torch.cos(4.0 * pi * normalized_time)
        return torch.stack(
            [
                relative_times,
                normalized_time,
                squared_time,
                cubic_time,
                sin_1,
                cos_1,
                sin_2,
                cos_2,
                sin_4,
            ][: self.config.time_feature_dim],
            dim=-1,
        )

    def project_tokens(self, features: torch.Tensor, relative_times: torch.Tensor) -> torch.Tensor:
        time_features = self.build_time_features(relative_times)
        token_inputs = torch.cat([features, time_features], dim=-1)
        hidden = self.input_projection(token_inputs)
        return self.temporal_frontend(hidden)

    def encode_projected_tokens(self, hidden: torch.Tensor) -> torch.Tensor:
        if self.position_embedding is not None:
            hidden = hidden + self.position_embedding(self.position_indices[1:].unsqueeze(0))
        return self.sequence_encoder(hidden)

    def forward(self, features: torch.Tensor, relative_times: torch.Tensor) -> torch.Tensor:
        return self.encode_projected_tokens(self.project_tokens(features, relative_times))


class FlipResetTransformer(nn.Module):
    def __init__(self, config: FlipResetTransformerConfig) -> None:
        super().__init__()
        self.config = config
        self.encoder = FlipResetEncoder(config)
        self.output_norm = nn.LayerNorm(config.model_dim)
        self.output_head = nn.Sequential(
            nn.Linear(config.model_dim, config.model_dim),
            nn.GELU(),
            nn.Dropout(config.dropout),
            nn.Linear(config.model_dim, 1),
        )

    def forward(self, features: torch.Tensor, relative_times: torch.Tensor) -> torch.Tensor:
        hidden = self.encoder(features, relative_times)
        sequence_hidden = self.output_norm(hidden)
        return self.output_head(sequence_hidden).squeeze(-1)


class MaskedFeatureReconstructionModel(nn.Module):
    def __init__(self, config: FlipResetTransformerConfig) -> None:
        super().__init__()
        self.config = config
        self.encoder = FlipResetEncoder(config)
        self.mask_token = nn.Parameter(torch.zeros(1, 1, config.model_dim))
        self.reconstruction_head = nn.Sequential(
            nn.LayerNorm(config.model_dim),
            nn.Linear(config.model_dim, config.model_dim),
            nn.GELU(),
            nn.Linear(config.model_dim, config.input_dim),
        )

    def forward(
        self,
        features: torch.Tensor,
        relative_times: torch.Tensor,
        masked_positions: torch.Tensor | None = None,
    ) -> torch.Tensor:
        time_features = self.encoder.build_time_features(relative_times)
        token_inputs = torch.cat([features, time_features], dim=-1)
        hidden = self.encoder.input_projection(token_inputs)
        if masked_positions is not None:
            hidden = torch.where(
                masked_positions.unsqueeze(-1),
                self.mask_token.expand(hidden.shape[0], hidden.shape[1], -1),
                hidden,
            )
        hidden = self.encoder.temporal_frontend(hidden)
        hidden = self.encoder.encode_projected_tokens(hidden)
        return self.reconstruction_head(hidden)
