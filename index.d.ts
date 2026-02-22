/* eslint-disable */
import type { CstNode, IrModule } from './types'
export * from './types'

export interface SourceEntry {
  source: string
  language: string
}

export type BatchParseResult = { success: true; result: CstNode } | { success: false; error: string }

export type BatchLowerResult = { success: true; result: IrModule } | { success: false; error: string }

export declare function lower(source: string, language: string): IrModule
export declare function lowerAsync(source: string, language: string): Promise<IrModule>
export declare function lowerBatch(entries: Array<SourceEntry>): Promise<Array<BatchLowerResult>>
export declare function parse(source: string, language: string): CstNode
export declare function parseAsync(source: string, language: string): Promise<CstNode>
export declare function parseBatch(entries: Array<SourceEntry>): Promise<Array<BatchParseResult>>
