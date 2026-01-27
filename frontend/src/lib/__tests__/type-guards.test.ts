/**
 * 类型守卫测试
 */

import {
  isValidSubstrateAddress,
  isValidBlockHash,
  isNonNegativeInteger,
  isValidBalanceString,
  parseEnumValue,
  parseBoolean,
  parseBigInt,
  parseNumber,
  parseString,
  parseOptionalString,
  parseBitmapToArray,
} from '../type-guards';

describe('Type Guards', () => {
  describe('isValidSubstrateAddress', () => {
    it('should validate correct SS58 addresses', () => {
      expect(isValidSubstrateAddress('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY')).toBe(true);
      expect(isValidSubstrateAddress('5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty')).toBe(true);
    });

    it('should reject invalid addresses', () => {
      expect(isValidSubstrateAddress('invalid')).toBe(false);
      expect(isValidSubstrateAddress('0x1234')).toBe(false);
      expect(isValidSubstrateAddress('')).toBe(false);
      expect(isValidSubstrateAddress(123 as any)).toBe(false);
    });
  });

  describe('isValidBlockHash', () => {
    it('should validate correct block hashes', () => {
      expect(isValidBlockHash('0x' + '0'.repeat(64))).toBe(true);
      expect(isValidBlockHash('0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890')).toBe(true);
    });

    it('should reject invalid block hashes', () => {
      expect(isValidBlockHash('0x123')).toBe(false);
      expect(isValidBlockHash('invalid')).toBe(false);
      expect(isValidBlockHash('')).toBe(false);
    });
  });

  describe('isNonNegativeInteger', () => {
    it('should validate non-negative integers', () => {
      expect(isNonNegativeInteger(0)).toBe(true);
      expect(isNonNegativeInteger(1)).toBe(true);
      expect(isNonNegativeInteger(1000)).toBe(true);
    });

    it('should reject negative numbers and non-integers', () => {
      expect(isNonNegativeInteger(-1)).toBe(false);
      expect(isNonNegativeInteger(1.5)).toBe(false);
      expect(isNonNegativeInteger(NaN)).toBe(false);
      expect(isNonNegativeInteger('123' as any)).toBe(false);
    });
  });

  describe('isValidBalanceString', () => {
    it('should validate balance strings', () => {
      expect(isValidBalanceString('1000')).toBe(true);
      expect(isValidBalanceString('1,000,000')).toBe(true);
      expect(isValidBalanceString('0')).toBe(true);
    });

    it('should reject invalid balance strings', () => {
      expect(isValidBalanceString('abc')).toBe(false);
      expect(isValidBalanceString('12.34.56')).toBe(false);
      expect(isValidBalanceString(123 as any)).toBe(false);
    });
  });

  describe('parseEnumValue', () => {
    const validValues = ['Active', 'Inactive', 'Pending'] as const;

    it('should parse string enum values', () => {
      expect(parseEnumValue('Active', validValues)).toBe('Active');
      expect(parseEnumValue('Pending', validValues)).toBe('Pending');
    });

    it('should parse object enum values', () => {
      expect(parseEnumValue({ Active: null }, validValues)).toBe('Active');
      expect(parseEnumValue({ Pending: null }, validValues)).toBe('Pending');
    });

    it('should return null for invalid values', () => {
      expect(parseEnumValue('Invalid', validValues)).toBeNull();
      expect(parseEnumValue({ Invalid: null }, validValues)).toBeNull();
      expect(parseEnumValue(123, validValues)).toBeNull();
    });
  });

  describe('parseBoolean', () => {
    it('should parse boolean values', () => {
      expect(parseBoolean(true)).toBe(true);
      expect(parseBoolean(false)).toBe(false);
    });

    it('should parse object boolean values', () => {
      expect(parseBoolean({ isTrue: true })).toBe(true);
      expect(parseBoolean({ isTrue: false })).toBe(false);
    });

    it('should return false for invalid values', () => {
      expect(parseBoolean('true')).toBe(false);
      expect(parseBoolean(1)).toBe(false);
      expect(parseBoolean(null)).toBe(false);
    });
  });

  describe('parseBigInt', () => {
    it('should parse bigint values', () => {
      expect(parseBigInt(BigInt(123))).toBe(BigInt(123));
      expect(parseBigInt(BigInt(0))).toBe(BigInt(0));
    });

    it('should parse number values', () => {
      expect(parseBigInt(123)).toBe(BigInt(123));
      expect(parseBigInt(0)).toBe(BigInt(0));
    });

    it('should parse string values', () => {
      expect(parseBigInt('123')).toBe(BigInt(123));
      expect(parseBigInt('1,000')).toBe(BigInt(1000));
    });

    it('should return 0 for invalid values', () => {
      expect(parseBigInt('invalid')).toBe(BigInt(0));
      expect(parseBigInt(null)).toBe(BigInt(0));
      expect(parseBigInt(undefined)).toBe(BigInt(0));
    });
  });

  describe('parseNumber', () => {
    it('should parse number values', () => {
      expect(parseNumber(123)).toBe(123);
      expect(parseNumber(0)).toBe(0);
      expect(parseNumber(-123)).toBe(-123);
    });

    it('should parse string values', () => {
      expect(parseNumber('123')).toBe(123);
      expect(parseNumber('1,000')).toBe(1000);
      expect(parseNumber('12.34')).toBe(12.34);
    });

    it('should return default value for invalid values', () => {
      expect(parseNumber('invalid')).toBe(0);
      expect(parseNumber('invalid', 999)).toBe(999);
      expect(parseNumber(null)).toBe(0);
    });
  });

  describe('parseString', () => {
    it('should parse string values', () => {
      expect(parseString('hello')).toBe('hello');
      expect(parseString('')).toBe('');
    });

    it('should convert non-string values', () => {
      expect(parseString(123)).toBe('123');
      expect(parseString(true)).toBe('true');
    });

    it('should return default value for null/undefined', () => {
      expect(parseString(null)).toBe('');
      expect(parseString(undefined)).toBe('');
      expect(parseString(null, 'default')).toBe('default');
    });
  });

  describe('parseOptionalString', () => {
    it('should parse non-empty strings', () => {
      expect(parseOptionalString('hello')).toBe('hello');
      expect(parseOptionalString('test')).toBe('test');
    });

    it('should return undefined for empty or invalid values', () => {
      expect(parseOptionalString('')).toBeUndefined();
      expect(parseOptionalString(null)).toBeUndefined();
      expect(parseOptionalString(undefined)).toBeUndefined();
      expect(parseOptionalString(123)).toBeUndefined();
    });
  });

  describe('parseBitmapToArray', () => {
    it('should parse bitmap to array of indices', () => {
      expect(parseBitmapToArray(0b0001)).toEqual([0]);
      expect(parseBitmapToArray(0b0010)).toEqual([1]);
      expect(parseBitmapToArray(0b0101)).toEqual([0, 2]);
      expect(parseBitmapToArray(0b1111)).toEqual([0, 1, 2, 3]);
    });

    it('should return empty array for 0', () => {
      expect(parseBitmapToArray(0)).toEqual([]);
    });

    it('should handle large bitmaps', () => {
      const bitmap = (1 << 0) | (1 << 5) | (1 << 10) | (1 << 15);
      expect(parseBitmapToArray(bitmap)).toEqual([0, 5, 10, 15]);
    });
  });
});
