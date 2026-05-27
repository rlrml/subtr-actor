use super::*;

goal_tag_node!(AerialGoalNode, AerialGoalCalculator, "aerial_goal");
goal_tag_node!(
    HighAerialGoalNode,
    HighAerialGoalCalculator,
    "high_aerial_goal"
);
goal_tag_node!(
    LongDistanceGoalNode,
    LongDistanceGoalCalculator,
    "long_distance_goal"
);
goal_tag_node!(OwnHalfGoalNode, OwnHalfGoalCalculator, "own_half_goal");
goal_tag_node!(EmptyNetGoalNode, EmptyNetGoalCalculator, "empty_net_goal");
goal_tag_node!(
    CounterAttackGoalNode,
    CounterAttackGoalCalculator,
    "counter_attack_goal"
);
