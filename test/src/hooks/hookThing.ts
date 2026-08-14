import { useSignal } from 'react';
export function useThing() { const [z] = useSignal(0); return z; }
