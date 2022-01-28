interface ArgMatch {
    /**
     * string if takes value
     * boolean if flag
     * string[] or null if takes multiple values
     */
    value: string | boolean | string[] | null;
    /**
     * Number of occurrences
     */
    occurrences: number;
}
interface SubcommandMatch {
    name: string;
    matches: CliMatches;
}
interface CliMatches {
    args: {
        [name: string]: ArgMatch;
    };
    subcommand: SubcommandMatch | null;
}
/**
 * Parse the arguments provided to the current process and get the matches using the configuration defined `tauri.conf.json > tauri > cli`.
 *
 * @returns A promise resolving to the parsed arguments.
 */
declare function getMatches(): Promise<CliMatches>;
export type { ArgMatch, SubcommandMatch, CliMatches };
export { getMatches };
