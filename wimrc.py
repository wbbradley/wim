# pylint: disable=C,W,R
from enum import Enum
from typing import Dict, List, Set

# from wim import (Direction, Mode, Newline, Noun, Rel, delete_to, join_lines, move, move_rel,
# switch_mode)


class Direction(Enum):
    LEFT = 'left'
    RIGHT = 'right'
    UP = 'up'
    DOWN = 'down'


class DKCommand:
    pass


class Mod(set):
    pass


class Noun(Mod, Enum):
    CHAR: Mod = {'char'}


class Rel(Mod, Enum):
    PRIOR = {'prior'}


DK = DKCommand | List['DK']


def move(_dir: Direction) -> DK:
    return DKCommand()


def move_rel(_props: Set[Mod]) -> DK:
    return DKCommand()


keymap: Dict[str, Dict[str, DK]] = {
    "normal": {
        "<Left>": move(Direction.LEFT),
        "<Right>": move(Direction.RIGHT),
        "<Up>": move(Direction.UP),
        "<Down>": move(Direction.DOWN),
        "h": move_rel(Noun.CHAR | Rel.PRIOR),
        "j": move(Direction.DOWN),
        "k": move(Direction.UP),
        "l": move(Direction.RIGHT),
        "J": join_lines,
        "o": [Newline.BELOW, switch_mode(Mode.INSERT)],
        "O": [Newline.ABOVE, move_rel(Noun.LINE | Rel.BEGIN)],
        "x": [delete_to(Noun.CHAR | Rel.NEXT)],

    }
}

# nmap <Left> :|left<CR>
# nmap <Right> :|right<CR>
# nmap <Up> :|up<CR>
# nmap <Down> :|down<CR>
# nmap b :|rel word prior<CR>
# nmap e :|rel word end<CR>
# nmap w :|rel word next<CR>
# nmap i :|mode insert<CR>
# nmap l :|right<CR>
# nmap h :|left<CR>
# nmap k :|up<CR>
# nmap j :|down<CR>
# nmap J :|join<CR>
# nmap o :|newline below<CR>:|mode insert<CR>
# nmap O :|newline above<CR>:|rel line begin<CR>:|mode insert<CR>
# nmap x :|delete char next<CR>
# nmap X :|delete char previous<CR>
