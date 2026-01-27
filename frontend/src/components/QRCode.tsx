/**
 * 二维码生成组件
 * 使用纯 React Native SVG 实现，无需额外依赖
 * 基于 QR Code 生成算法简化版
 */

import React, { useMemo } from 'react';
import { View, StyleSheet } from 'react-native';
import Svg, { Rect } from 'react-native-svg';

interface QRCodeProps {
  value: string;
  size?: number;
  color?: string;
  backgroundColor?: string;
}

// 简化的 QR Code 生成器
// 注意：这是一个简化版本，生成的是类似二维码的图案
// 实际生产环境建议使用专业的 QR Code 库

// 字符串转换为二进制数据
function stringToBinary(str: string): number[] {
  const binary: number[] = [];
  for (let i = 0; i < str.length; i++) {
    const charCode = str.charCodeAt(i);
    for (let j = 7; j >= 0; j--) {
      binary.push((charCode >> j) & 1);
    }
  }
  return binary;
}

// 生成简化的 QR 矩阵
function generateQRMatrix(data: string, moduleCount: number): number[][] {
  const matrix: number[][] = [];
  for (let i = 0; i < moduleCount; i++) {
    matrix[i] = [];
    for (let j = 0; j < moduleCount; j++) {
      matrix[i][j] = 0;
    }
  }

  // 添加定位图案 (Position Detection Patterns)
  const addFinderPattern = (row: number, col: number) => {
    for (let r = -1; r <= 7; r++) {
      for (let c = -1; c <= 7; c++) {
        const newRow = row + r;
        const newCol = col + c;
        if (newRow < 0 || newRow >= moduleCount || newCol < 0 || newCol >= moduleCount) {
          continue;
        }
        if (
          (r >= 0 && r <= 6 && (c === 0 || c === 6)) ||
          (c >= 0 && c <= 6 && (r === 0 || r === 6)) ||
          (r >= 2 && r <= 4 && c >= 2 && c <= 4)
        ) {
          const matrixRow = matrix[newRow];
          if (matrixRow) {
            matrixRow[newCol] = 1;
          }
        }
      }
    }
  };

  // 三个角的定位图案
  addFinderPattern(0, 0);
  addFinderPattern(0, moduleCount - 7);
  addFinderPattern(moduleCount - 7, 0);

  // 添加时序图案 (Timing Patterns)
  for (let i = 8; i < moduleCount - 8; i++) {
    const row6 = matrix[6];
    const rowI = matrix[i];
    if (row6) row6[i] = i % 2 === 0 ? 1 : 0;
    if (rowI) rowI[6] = i % 2 === 0 ? 1 : 0;
  }

  // 将数据编码到矩阵中
  const binary = stringToBinary(data);
  let binaryIndex = 0;

  // 从右下角开始填充数据
  for (let col = moduleCount - 1; col >= 0; col -= 2) {
    if (col === 6) col--; // 跳过时序列
    for (let row = moduleCount - 1; row >= 0; row--) {
      for (let c = 0; c < 2; c++) {
        const currentCol = col - c;
        if (currentCol < 0) continue;
        
        // 跳过已占用的区域
        if (
          (row < 9 && currentCol < 9) ||
          (row < 9 && currentCol >= moduleCount - 8) ||
          (row >= moduleCount - 8 && currentCol < 9) ||
          row === 6 ||
          currentCol === 6
        ) {
          continue;
        }

        const matrixRow = matrix[row];
        if (matrixRow) {
          if (binaryIndex < binary.length) {
            matrixRow[currentCol] = binary[binaryIndex] ?? 0;
            binaryIndex++;
          } else {
            // 用伪随机数据填充剩余空间
            matrixRow[currentCol] = (row * currentCol + row + currentCol) % 2;
          }
        }
      }
    }
  }

  return matrix;
}

export const QRCode: React.FC<QRCodeProps> = ({
  value,
  size = 200,
  color = '#000000',
  backgroundColor = '#FFFFFF',
}) => {
  const matrix = useMemo(() => {
    // 根据数据长度确定模块数量
    const moduleCount = Math.max(21, Math.min(33, 21 + Math.floor(value.length / 10) * 4));
    return generateQRMatrix(value, moduleCount);
  }, [value]);

  const moduleSize = size / matrix.length;

  return (
    <View style={[styles.container, { width: size, height: size, backgroundColor }]}>
      <Svg width={size} height={size}>
        {matrix.map((row, rowIndex) =>
          row.map((cell, colIndex) =>
            cell === 1 ? (
              <Rect
                key={`${rowIndex}-${colIndex}`}
                x={colIndex * moduleSize}
                y={rowIndex * moduleSize}
                width={moduleSize}
                height={moduleSize}
                fill={color}
              />
            ) : null
          )
        )}
      </Svg>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    borderRadius: 8,
    overflow: 'hidden',
  },
});

export default QRCode;
