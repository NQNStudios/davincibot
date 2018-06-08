import {Idea} from "../Idea";
import {BotProcess, BotStatus} from "../DaVinciBot";
import {AddIdeaProcess} from "./AddIdeaProcess";

// TODO processes should be able to handleInput in a standard form where
// different options are either a number or a case-insensitive string.
// There should also be a standard form for outputting these options as is done
// below

// TODO should process descriptions be markdown-enabled? With options available
// on getOutput() to either ignore or parse formatting into another form the
// client chooses

export class PlanIdeaProcess extends BotProcess
{
    start(): void {
        this.status = BotStatus.HasOutput;
    }

    getOutput(): string {
        this.status = BotStatus.NeedsInput;
        return `Planning for idea ${this.rootIdea.name}:
            1. [Break] this idea into smaller ideas.
            2. Define a [time] estimate for this idea.
            3. [Describe] this idea. (Current description: ${this.rootIdea.description})
            4. [quit]
        `;

    }

    handleInput(input:string) {
        this.status = BotStatus.HasOutput;
        switch (input.toLowerCase()) {
            case '1':
            case 'break':
                this.bot.startProcess(new AddIdeaProcess(this.bot, this.rootIdea));
                break;
            case '2':
            case 'time':
                // TODO start a TimeIdeaProcess on rootIdea
                break;
            case '3':
            case 'describe':
                // TODO start an add description process on rootIdea
                break;
            case '4':
            case 'quit':
                this.status = BotStatus.Idle;
                break;
        }
    }
}
