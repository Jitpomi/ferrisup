#!/usr/bin/env python3
"""
Template Visualizer for Burn Templates
Creates simple diagrams to visualize the structure and data flow of the templates
"""

import os
import sys
from graphviz import Digraph

def create_image_classifier_diagram():
    """Create a diagram for the Image Classifier template"""
    dot = Digraph(comment='Image Classifier Architecture')
    
    # Set graph attributes
    dot.attr('graph', rankdir='TB', size='8,8', dpi='300')
    dot.attr('node', shape='box', style='filled', color='lightblue', fontname='Arial')
    dot.attr('edge', fontname='Arial')
    
    # Add nodes
    dot.node('config', 'Configuration\n(config.rs)', shape='note', color='lightgreen')
    dot.node('data', 'Data Processing\n(data.rs)', shape='box')
    dot.node('model', 'CNN Model\n(model.rs)', shape='box')
    dot.node('main', 'Main Application\n(main.rs)', shape='box')
    
    # Add subgraph for data flow
    with dot.subgraph(name='cluster_dataflow') as c:
        c.attr(label='Data Flow', style='dashed')
        c.node('images', 'Input Images', shape='folder', color='lightyellow')
        c.node('tensors', 'Image Tensors', shape='box3d', color='lightgrey')
        c.node('features', 'Feature Maps', shape='box3d', color='lightgrey')
        c.node('predictions', 'Class Predictions', shape='box3d', color='lightgrey')
        
        c.edge('images', 'tensors')
        c.edge('tensors', 'features')
        c.edge('features', 'predictions')
    
    # Add edges
    dot.edge('config', 'data', label='Parameters')
    dot.edge('config', 'model', label='Architecture')
    dot.edge('config', 'main', label='Training Settings')
    
    dot.edge('data', 'tensors', label='Process')
    dot.edge('model', 'features', label='Extract')
    dot.edge('main', 'predictions', label='Output')
    
    # Save the diagram
    dot.render('burn-image-classifier-diagram', format='png', cleanup=True)
    print("Created image classifier diagram: burn-image-classifier-diagram.png")

def create_text_analyzer_diagram():
    """Create a diagram for the Text Analyzer template"""
    dot = Digraph(comment='Text Analyzer Architecture')
    
    # Set graph attributes
    dot.attr('graph', rankdir='TB', size='8,8', dpi='300')
    dot.attr('node', shape='box', style='filled', color='lightblue', fontname='Arial')
    dot.attr('edge', fontname='Arial')
    
    # Add nodes
    dot.node('config', 'Configuration\n(config.rs)', shape='note', color='lightgreen')
    dot.node('data', 'Text Processing\n(data.rs)', shape='box')
    dot.node('model', 'LSTM Model\n(model.rs)', shape='box')
    dot.node('main', 'Main Application\n(main.rs)', shape='box')
    
    # Add subgraph for data flow
    with dot.subgraph(name='cluster_dataflow') as c:
        c.attr(label='Data Flow', style='dashed')
        c.node('text', 'Input Text', shape='folder', color='lightyellow')
        c.node('tokens', 'Token Sequences', shape='box3d', color='lightgrey')
        c.node('embeddings', 'Word Embeddings', shape='box3d', color='lightgrey')
        c.node('sentiment', 'Sentiment Prediction', shape='box3d', color='lightgrey')
        
        c.edge('text', 'tokens')
        c.edge('tokens', 'embeddings')
        c.edge('embeddings', 'sentiment')
    
    # Add edges
    dot.edge('config', 'data', label='Parameters')
    dot.edge('config', 'model', label='Architecture')
    dot.edge('config', 'main', label='Training Settings')
    
    dot.edge('data', 'tokens', label='Tokenize')
    dot.edge('model', 'embeddings', label='Embed')
    dot.edge('main', 'sentiment', label='Output')
    
    # Save the diagram
    dot.render('burn-text-analyzer-diagram', format='png', cleanup=True)
    print("Created text analyzer diagram: burn-text-analyzer-diagram.png")

def create_data_predictor_diagram():
    """Create a diagram for the Data Predictor template"""
    dot = Digraph(comment='Data Predictor Architecture')
    
    # Set graph attributes
    dot.attr('graph', rankdir='TB', size='8,8', dpi='300')
    dot.attr('node', shape='box', style='filled', color='lightblue', fontname='Arial')
    dot.attr('edge', fontname='Arial')
    
    # Add nodes
    dot.node('config', 'Configuration\n(config.rs)', shape='note', color='lightgreen')
    dot.node('data', 'Data Processing\n(data.rs)', shape='box')
    dot.node('model', 'Neural Network\n(model.rs)', shape='box')
    dot.node('main', 'Main Application\n(main.rs)', shape='box')
    
    # Add subgraph for data flow
    with dot.subgraph(name='cluster_dataflow') as c:
        c.attr(label='Data Flow', style='dashed')
        c.node('csv', 'CSV Data', shape='folder', color='lightyellow')
        c.node('features', 'Numerical Features', shape='box3d', color='lightgrey')
        c.node('normalized', 'Normalized Features', shape='box3d', color='lightgrey')
        c.node('prediction', 'Value Prediction', shape='box3d', color='lightgrey')
        
        c.edge('csv', 'features')
        c.edge('features', 'normalized')
        c.edge('normalized', 'prediction')
    
    # Add edges
    dot.edge('config', 'data', label='Parameters')
    dot.edge('config', 'model', label='Architecture')
    dot.edge('config', 'main', label='Training Settings')
    
    dot.edge('data', 'normalized', label='Normalize')
    dot.edge('model', 'prediction', label='Predict')
    dot.edge('main', 'prediction', label='Output')
    
    # Save the diagram
    dot.render('burn-data-predictor-diagram', format='png', cleanup=True)
    print("Created data predictor diagram: burn-data-predictor-diagram.png")

def create_templates_overview():
    """Create an overview diagram of all templates"""
    dot = Digraph(comment='Burn Templates Overview')
    
    # Set graph attributes
    dot.attr('graph', rankdir='LR', size='10,8', dpi='300')
    dot.attr('node', fontname='Arial')
    dot.attr('edge', fontname='Arial')
    
    # Add central node
    dot.node('burn', 'Burn Framework', shape='box', style='filled', color='orange')
    
    # Add template nodes
    dot.node('image', 'Image Classifier', shape='box', style='filled', color='skyblue')
    dot.node('text', 'Text Analyzer', shape='box', style='filled', color='lightgreen')
    dot.node('data', 'Data Predictor', shape='box', style='filled', color='lightpink')
    
    # Add feature nodes for Image Classifier
    with dot.subgraph(name='cluster_image') as c:
        c.attr(label='Image Classifier Features', style='dashed')
        c.node('cnn', 'CNN Architecture', shape='ellipse')
        c.node('augment', 'Image Augmentation', shape='ellipse')
        c.node('classify', 'Multi-class Classification', shape='ellipse')
    
    # Add feature nodes for Text Analyzer
    with dot.subgraph(name='cluster_text') as c:
        c.attr(label='Text Analyzer Features', style='dashed')
        c.node('lstm', 'LSTM Architecture', shape='ellipse')
        c.node('embed', 'Word Embeddings', shape='ellipse')
        c.node('sentiment', 'Sentiment Analysis', shape='ellipse')
    
    # Add feature nodes for Data Predictor
    with dot.subgraph(name='cluster_data') as c:
        c.attr(label='Data Predictor Features', style='dashed')
        c.node('mlp', 'MLP Architecture', shape='ellipse')
        c.node('normalize', 'Data Normalization', shape='ellipse')
        c.node('regression', 'Regression Analysis', shape='ellipse')
    
    # Connect templates to Burn
    dot.edge('burn', 'image')
    dot.edge('burn', 'text')
    dot.edge('burn', 'data')
    
    # Connect features to templates
    dot.edge('image', 'cnn')
    dot.edge('image', 'augment')
    dot.edge('image', 'classify')
    
    dot.edge('text', 'lstm')
    dot.edge('text', 'embed')
    dot.edge('text', 'sentiment')
    
    dot.edge('data', 'mlp')
    dot.edge('data', 'normalize')
    dot.edge('data', 'regression')
    
    # Save the diagram
    dot.render('burn-templates-overview', format='png', cleanup=True)
    print("Created templates overview: burn-templates-overview.png")

def main():
    """Main function to create all diagrams"""
    try:
        # Create output directory
        os.makedirs('diagrams', exist_ok=True)
        os.chdir('diagrams')
        
        # Create all diagrams
        create_image_classifier_diagram()
        create_text_analyzer_diagram()
        create_data_predictor_diagram()
        create_templates_overview()
        
        print("All diagrams created successfully in the 'diagrams' directory")
    except Exception as e:
        print(f"Error creating diagrams: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main())
