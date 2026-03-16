from __future__ import annotations

import unittest

import torch

from flip_reset_transformer.model import FlipResetTransformer, FlipResetTransformerConfig
from flip_reset_transformer.train import TolerantSequenceLoss


class ModelAndLossTests(unittest.TestCase):
    def test_transformer_outputs_sequence_logits(self) -> None:
        model = FlipResetTransformer(
            FlipResetTransformerConfig(
                input_dim=6,
                window_length=5,
                model_dim=16,
                cnn_layers=1,
                cnn_kernel_size=3,
                encoder_layers=1,
                attention_heads=4,
                feedforward_dim=32,
                dropout=0.0,
            )
        )
        features = torch.randn(2, 5, 6)
        relative_times = torch.linspace(-0.1, 0.1, 5).repeat(2, 1)
        logits = model(features, relative_times)
        self.assertEqual(tuple(logits.shape), (2, 5))

    def test_transformer_outputs_sequence_logits_without_additive_position_embedding(self) -> None:
        model = FlipResetTransformer(
            FlipResetTransformerConfig(
                input_dim=6,
                window_length=5,
                model_dim=16,
                time_feature_dim=9,
                cnn_layers=1,
                cnn_kernel_size=3,
                encoder_layers=1,
                attention_heads=4,
                feedforward_dim=32,
                dropout=0.0,
                additive_position_embedding=False,
            )
        )
        features = torch.randn(2, 5, 6)
        relative_times = torch.linspace(-0.1, 0.1, 5).repeat(2, 1)
        logits = model(features, relative_times)
        self.assertEqual(tuple(logits.shape), (2, 5))

    def test_tcn_outputs_sequence_logits(self) -> None:
        model = FlipResetTransformer(
            FlipResetTransformerConfig(
                input_dim=6,
                window_length=5,
                model_dim=16,
                encoder_type="tcn",
                cnn_layers=1,
                cnn_kernel_size=3,
                encoder_layers=3,
                attention_heads=4,
                feedforward_dim=32,
                dropout=0.0,
                additive_position_embedding=False,
            )
        )
        features = torch.randn(2, 5, 6)
        relative_times = torch.linspace(-0.1, 0.1, 5).repeat(2, 1)
        logits = model(features, relative_times)
        self.assertEqual(tuple(logits.shape), (2, 5))

    def test_tolerant_sequence_loss_prefers_near_miss_to_far_miss(self) -> None:
        loss_fn = TolerantSequenceLoss(
            radius=2,
            cover_loss_weight=1.0,
            false_positive_loss_weight=1.0,
            count_loss_weight=0.25,
            exact_bce_loss_weight=0.0,
            exact_positive_weight=1.0,
        )
        labels = torch.tensor([[0.0, 0.0, 1.0, 0.0, 0.0]], dtype=torch.float32)
        near_logits = torch.tensor([[-6.0, -6.0, -6.0, 6.0, -6.0]], dtype=torch.float32)
        far_logits = torch.tensor([[6.0, -6.0, -6.0, -6.0, -6.0]], dtype=torch.float32)
        exact_logits = torch.tensor([[-6.0, -6.0, 6.0, -6.0, -6.0]], dtype=torch.float32)

        near_loss = float(loss_fn(near_logits, labels).item())
        far_loss = float(loss_fn(far_logits, labels).item())
        exact_loss = float(loss_fn(exact_logits, labels).item())

        self.assertLess(exact_loss, near_loss)
        self.assertLess(near_loss, far_loss)


if __name__ == "__main__":
    unittest.main()
