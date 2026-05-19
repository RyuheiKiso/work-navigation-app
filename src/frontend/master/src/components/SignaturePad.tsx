import type React from 'react';
import { useRef, useEffect, useState, useCallback } from 'react';
import { Box, Button, Stack, Typography } from '@mui/material';

// canvas ベースの手書きサイン UI。base64 PNG として親に通知し、サーバ側でハッシュチェーンに紐付ける（FR-MA-009）。
export function SignaturePad({
  onChange,
  width = 480,
  height = 200,
}: {
  onChange: (signatureBase64: string | null) => void;
  width?: number;
  height?: number;
}): React.ReactElement {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const drawingRef = useRef(false);
  const [hasStrokes, setHasStrokes] = useState(false);

  const getContext = useCallback((): CanvasRenderingContext2D | null => {
    const canvas = canvasRef.current;
    if (!canvas) return null;
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;
    ctx.lineWidth = 2;
    ctx.lineCap = 'round';
    ctx.strokeStyle = '#0F172A';
    return ctx;
  }, []);

  useEffect(() => {
    const ctx = getContext();
    if (!ctx) return;
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(0, 0, width, height);
  }, [width, height, getContext]);

  const pointerPos = (e: React.PointerEvent<HTMLCanvasElement>): { x: number; y: number } => {
    const rect = e.currentTarget.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  };

  const handleDown = (e: React.PointerEvent<HTMLCanvasElement>): void => {
    drawingRef.current = true;
    const ctx = getContext();
    if (!ctx) return;
    const { x, y } = pointerPos(e);
    ctx.beginPath();
    ctx.moveTo(x, y);
  };

  const handleMove = (e: React.PointerEvent<HTMLCanvasElement>): void => {
    if (!drawingRef.current) return;
    const ctx = getContext();
    if (!ctx) return;
    const { x, y } = pointerPos(e);
    ctx.lineTo(x, y);
    ctx.stroke();
    setHasStrokes(true);
  };

  const handleUp = (): void => {
    drawingRef.current = false;
    const canvas = canvasRef.current;
    if (!canvas) return;
    onChange(canvas.toDataURL('image/png'));
  };

  const clear = (): void => {
    const ctx = getContext();
    if (!ctx) return;
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(0, 0, width, height);
    setHasStrokes(false);
    onChange(null);
  };

  return (
    <Stack spacing={1}>
      <Typography variant="caption" color="text.secondary">
        以下の枠内に署名してください
      </Typography>
      <Box
        sx={{
          border: '2px solid',
          borderColor: hasStrokes ? 'success.main' : 'divider',
          borderRadius: 1,
          touchAction: 'none',
          width,
          height,
          backgroundColor: '#FFFFFF',
        }}
      >
        <canvas
          ref={canvasRef}
          width={width}
          height={height}
          onPointerDown={handleDown}
          onPointerMove={handleMove}
          onPointerUp={handleUp}
          onPointerLeave={handleUp}
          aria-label="署名入力エリア"
          role="img"
        />
      </Box>
      <Button onClick={clear} size="small" aria-label="署名をクリア">
        クリア
      </Button>
    </Stack>
  );
}
