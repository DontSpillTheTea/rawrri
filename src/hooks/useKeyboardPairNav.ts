import { useEffect } from "react";

interface PairNavArgs {
  canNavigate: boolean;
  onNext: () => void;
  onPrev: () => void;
}

export function useKeyboardPairNav({ canNavigate, onNext, onPrev }: PairNavArgs) {
  useEffect(() => {
    if (!canNavigate) return;

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "ArrowDown" || event.key.toLowerCase() === "j") {
        event.preventDefault();
        onNext();
      } else if (event.key === "ArrowUp" || event.key.toLowerCase() === "k") {
        event.preventDefault();
        onPrev();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [canNavigate, onNext, onPrev]);
}
