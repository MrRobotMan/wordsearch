from __future__ import annotations

import enum
import random
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterator, NamedTuple, Optional

from colorama import Fore


class Location(NamedTuple):
    row: int
    column: int


class Direction(enum.Enum):
    UP = "UP"
    DOWN = "DOWN"
    LEFT = "RIGHT to LEFT"
    RIGHT = "LEFT to RIGHT"
    ANGLED_UP_RIGHT = "UP to the RIGHT"
    ANGLED_DOWN_RIGHT = "DOWN to the RIGHT"
    ANGLED_UP_LEFT = "UP to the LEFT"
    ANGLED_DOWN_LEFT = "DOWN to the LEFT"


@dataclass(frozen=True, slots=True)
class Grid:
    text: tuple[str, ...]
    columns: list[str] = field(default_factory=list, init=False, repr=False)
    diag_up_right: list[str] = field(default_factory=list, init=False, repr=False)
    diag_down_right: list[str] = field(default_factory=list, init=False, repr=False)
    highlighted: list[list[str]] = field(default_factory=list, init=False, repr=False)

    def __post_init__(self):
        row_count = len(self.text)
        col_count = len(self.text[0])
        cols: list[list[str]] = [[] for _ in range(col_count)]
        diag_up: list[list[str]] = [[] for _ in range(row_count + col_count - 1)]
        diag_dn: list[list[str]] = [[] for _ in range(row_count + col_count - 1)]
        for row, line in enumerate(self.text):
            self.highlighted.append([])
            for col, letter in enumerate(line):
                cols[col].append(letter)
                diag_up[row + col].append(letter)
                diag_dn[col - row + row_count - 1].append(letter)
                self.highlighted[row].append(letter)
        for col in cols:
            self.columns.append("".join(col))
        for row in diag_up:
            self.diag_up_right.append("".join(row[::-1]))
        for row in diag_dn:
            self.diag_down_right.append("".join(row))

    @classmethod
    def from_str(cls, text: str) -> Grid:
        return cls(tuple(text.replace(" ", "").splitlines()))

    def highlight(
        self, start: Location, direction: Direction, length: int, color: str
    ) -> None:
        """Set the indices of the highlighted text"""
        match direction:
            case Direction.RIGHT:
                row_off, col_off = (0, 1)
            case Direction.LEFT:
                row_off, col_off = (0, -1)
            case Direction.UP:
                row_off, col_off = (-1, 0)
            case Direction.DOWN:
                row_off, col_off = (1, 0)
            case Direction.ANGLED_UP_RIGHT:
                row_off, col_off = (-1, 1)
            case Direction.ANGLED_DOWN_LEFT:
                row_off, col_off = (1, -1)
            case Direction.ANGLED_UP_LEFT:
                row_off, col_off = (-1, -1)
            case Direction.ANGLED_DOWN_RIGHT:
                row_off, col_off = (1, 1)
        for ind in range(length):
            row, col = start.row + ind * row_off, start.column + ind * col_off
            letter = self.text[row][col]
            self.highlighted[row][col] = f"{color}{letter}{Fore.RESET}"

    def colored(self) -> str:
        """Print the text with highlighting."""
        return "\n".join((" ".join(row) for row in self.highlighted))

    def __str__(self) -> str:
        return "\n".join((" ".join(row) for row in self.text))

    def find_word(self, word: str, color: str) -> Optional[tuple[Location, Direction]]:
        """Finds a word in the grid.

        Returns
            - the location (row, column) of the first letter
            - the direction of the word from the location of the first letter
        """
        # Search through rows.
        for row, text in enumerate(self.text):
            result = find_in_group(word, text)
            if result:
                col, to_right = result
                direction = Direction.RIGHT if to_right else Direction.LEFT
                start = Location(row, col)
                self.highlight(start, direction, len(word), color)
                return (start, direction)

        # Search through columns.
        for col, text in enumerate(self.columns):
            result = find_in_group(word, text)
            if result:
                row, down = result
                direction = Direction.DOWN if down else Direction.UP
                start = Location(row, col)
                self.highlight(start, direction, len(word), color)
                return (start, direction)

        num_rows = len(self.text)
        # Search diagonal up-right ("rows" are top left to bottom right)
        for diag, text in enumerate(self.diag_up_right):
            result = find_in_group(word, text)
            if result:
                index, forward = result
                direction = (
                    Direction.ANGLED_UP_RIGHT if forward else Direction.ANGLED_DOWN_LEFT
                )
                if diag < num_rows:
                    row = diag - index
                    col = index
                else:
                    row = num_rows - index - 1
                    col = (diag - num_rows) + index + 1
                start = Location(row, col)
                self.highlight(start, direction, len(word), color)
                return (start, direction)

        # Search diagonal down-right ("rows" bottom left to top right)
        for diag, text in enumerate(self.diag_down_right):
            result = find_in_group(word, text)
            if result:
                index, forward = result
                direction = (
                    Direction.ANGLED_DOWN_RIGHT if forward else Direction.ANGLED_UP_LEFT
                )
                if diag < num_rows:
                    row = num_rows - diag + index - 1
                    col = index
                else:
                    row = index
                    col = diag - num_rows + index + 1
                start = Location(row, col)
                self.highlight(start, direction, len(word), color)
                return (start, direction)


def read_file(file: Path) -> tuple[Grid, Iterator[str]]:
    """Get the words to find in the grid and the grid of letters

    The file should be formatted with the grid of letters on top and at least 1
    blank line between the grid and the words to find.
    """
    text = file.read_text()
    grid, words = text.split("\n\n")
    grid = Grid.from_str(grid)
    words = get_words(words)
    return grid, words


def get_words(text: str) -> Iterator[str]:
    """Parse the word list into words

    example:
        >>> tuple(get_words("\\n BATHROOM \\n  FLUSH      WIPE  "))
        ('BATHROOM', 'FLUSH', 'WIPE')
    """
    words = re.split(r"\s+", text.strip())
    return (word for word in words)


def find_in_group(word: str, row: str) -> Optional[tuple[int, bool]]:
    """Finds if a word in the the group.

    If the word is found the index and a boolen (forward, backward) is returned.

    examples:
        >>> find_in_group("BATHROOM", "RRXIEWFUFPURLWLMOORHTABS")
        (22, False)
        >>> find_in_group("SEAT", "XSEATHSULFUSGLPNZUUMLEMJ")
        (1, True)
    """
    if word in row:
        return (row.find(word), True)
    if word in row[::-1]:
        last_index = len(row) - 1
        last_col = row[::-1].find(word)
        return (last_index - last_col, False)


def useable_colors() -> tuple[str]:
    bad_colors = ["BLACK", "WHITE", "LIGHTBLACK_EX", "LIGHTWHITE_EX", "RESET", "BLUE"]
    all = vars(Fore)
    colors = (all[color] for color in all if color not in bad_colors)
    return tuple(colors)


def main() -> None:
    if len(sys.argv) == 1:
        file = Path(input("Enter the input file: "))
    else:
        file = Path(sys.argv[1])
    colors = useable_colors()
    grid, words = read_file(file)
    print(grid)
    input("Press 'ENTER' to reveal solution")
    for word in words:
        color = random.choice(colors)
        found = grid.find_word(word, color)
        if found:
            print(
                f"Found {color}{word}{Fore.RESET} at {found[0].row}, {found[0].column} going {found[1].value}"
            )
        else:
            print(f"Did not find {word}")
    print(grid.colored())


if __name__ == "__main__":
    main()
