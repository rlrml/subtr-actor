from . import test_directory

import subtr_actor


def test_parse_replay():
    subtr_actor.get_ndarray_with_info_from_replay_filepath(os.path.join(
        test_directory, "029103f9-4d58-4964-b47a-539b32f6fb33.replay"
    ))
