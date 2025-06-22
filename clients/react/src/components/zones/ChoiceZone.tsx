import { Button } from '../ui';

interface Choice {
  id: string;
  label: string;
  linkedEntities?: string[];
}

interface ChoiceZoneProps {
  zone: {
    id: string;
    type: 'choice';
    items?: Choice[];
    prompt?: string;
    visibility?: string;
  };
  onSelect: (choice: string) => void;
  isActive?: boolean;
  className?: string;
  multiStepState?: any;
}

export function ChoiceZone({ zone, onSelect, isActive = true, className = '', multiStepState }: ChoiceZoneProps) {
  const choices = zone.items || [];
  
  if (!isActive || choices.length === 0) {
    return null;
  }

  return (
    <div className={`choice-zone p-4 bg-gray-100 rounded-lg relative ${className}`} data-testid="choice-zone">
      {zone.prompt && (
        <div className="prompt mb-3 text-lg font-medium text-gray-800">
          {zone.prompt}
        </div>
      )}
      
      <div className="choices grid gap-2">
        {choices.map((choice) => (
          <Button
            key={choice.id}
            onClick={() => onSelect(choice.id)}
            variant="secondary"
            className="choice-button p-3 text-left hover:bg-blue-50 border-2 hover:border-blue-300 transition-colors"
          >
            <div className="choice-content">
              <div className="choice-label font-medium">
                {choice.label}
              </div>
              {choice.linkedEntities && choice.linkedEntities.length > 0 && (
                <div className="linked-entities text-sm text-gray-600 mt-1">
                  Related: {choice.linkedEntities.join(', ')}
                </div>
              )}
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}

// Helper component for conceptual choices (like rank selection)
interface ConceptualChoiceProps {
  options: string[];
  prompt: string;
  onSelect: (option: string) => void;
  className?: string;
}

export function ConceptualChoice({ options, prompt, onSelect, className = '' }: ConceptualChoiceProps) {
  return (
    <div className={`conceptual-choice p-4 bg-blue-50 rounded-lg border-2 border-blue-200 ${className}`}>
      <div className="prompt mb-3 text-lg font-medium text-blue-800">
        {prompt}
      </div>
      
      <div className="options grid grid-cols-4 gap-2">
        {options.map((option) => (
          <Button
            key={option}
            onClick={() => onSelect(option)}
            variant="secondary"
            className="option-button p-2 text-center hover:bg-blue-100 border border-blue-300 hover:border-blue-500 transition-colors"
          >
            {option}
          </Button>
        ))}
      </div>
    </div>
  );
}

// Helper component for diegetic choices (like choosing from visible cards)
interface DiegeticChoiceProps {
  entities: Array<{
    id: string;
    name?: string;
    props?: Record<string, any>;
  }>;
  prompt: string;
  onSelect: (entityId: string) => void;
  className?: string;
}

export function DiegeticChoice({ entities, prompt, onSelect, className = '' }: DiegeticChoiceProps) {
  return (
    <div className={`diegetic-choice p-4 bg-green-50 rounded-lg border-2 border-green-200 ${className}`}>
      <div className="prompt mb-3 text-lg font-medium text-green-800">
        {prompt}
      </div>
      
      <div className="entities grid gap-2">
        {entities.map((entity) => (
          <Button
            key={entity.id}
            onClick={() => onSelect(entity.id)}
            variant="secondary"
            className="entity-button p-3 text-left hover:bg-green-100 border border-green-300 hover:border-green-500 transition-colors"
          >
            <div className="entity-content">
              <div className="entity-name font-medium">
                {entity.name || entity.id}
              </div>
              {entity.props && (
                <div className="entity-props text-sm text-gray-600 mt-1">
                  {Object.entries(entity.props).map(([key, value]) => (
                    <span key={key} className="prop mr-2">
                      {key}: {value}
                    </span>
                  ))}
                </div>
              )}
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}