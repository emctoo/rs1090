from typing import Sequence

def decode_one(msg: str) -> list[int]: ...
def decode_vec(msgs: Sequence[str]) -> list[int]: ...
def decode_vec_time(
    msgs: Sequence[str],
    ts: Sequence[float],
    position: None | tuple[float, float] = None,
) -> list[int]: ...
def decode_parallel(msgs: Sequence[Sequence[str]]) -> list[int]: ...
def decode_parallel_time(
    msgs: Sequence[Sequence[str]],
    ts: Sequence[Sequence[float]],
    position: None | tuple[float, float] = None,
) -> list[int]: ...
