/* eslint-disable */
import type { CstNode, IrModule } from './types'
export * from './types'

export declare function lower(source: string, language: string): IrModule
export declare function lowerAsync(source: string, language: string): Promise<IrModule>
export declare function parse(source: string, language: string): CstNode
export declare function parseAsync(source: string, language: string): Promise<CstNode>
