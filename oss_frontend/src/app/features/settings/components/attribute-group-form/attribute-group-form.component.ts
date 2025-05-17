import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { AttributeGroupService } from '../../../../shared/services/attribute-group.service';
import { RadiusAttributeService } from '../../../../shared/services/radius-attribute.service';
import { AttributeGroup } from '../../../../shared/models/attribute-group.model';
import { RadiusAttribute, AttributeType } from '../../../../shared/models/radius-attribute.model';

@Component({
  selector: 'app-attribute-group-form',
  templateUrl: './attribute-group-form.component.html',
  styleUrls: ['./attribute-group-form.component.scss']
})
export class AttributeGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  attributeForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error: string | null = null;
  attributes: RadiusAttribute[] = [];
  attributeTypes: AttributeType[] = ['string', 'integer', 'ipaddr', 'date', 'octets'];

  constructor(
    private fb: FormBuilder,
    private attributeGroupService: AttributeGroupService,
    private radiusAttributeService: RadiusAttributeService,
    private route: ActivatedRoute,
    private router: Router
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: ['']
    });

    this.attributeForm = this.fb.group({
      vendor_id: [0, [Validators.required, Validators.min(0)]],
      attribute_id: ['', [Validators.required, Validators.min(1)]],
      attribute_name: ['', [Validators.required, Validators.maxLength(255)]],
      attribute_type: ['string', Validators.required],
      attribute_value: ['', [Validators.required, Validators.maxLength(255)]]
    });
  }

  ngOnInit(): void {
    const id = this.route.snapshot.paramMap.get('id');
    if (id) {
      this.isEditMode = true;
      this.loadGroup(parseInt(id));
    }
  }

  loadGroup(id: number): void {
    this.loading = true;
    this.attributeGroupService.getGroup(id).subscribe({
      next: (group) => {
        this.groupForm.patchValue({
          name: group.name,
          description: group.description
        });
        this.loadAttributes(id);
        this.loading = false;
      },
      error: (error) => {
        this.error = 'Failed to load group';
        this.loading = false;
      }
    });
  }

  loadAttributes(groupId: number): void {
    this.radiusAttributeService.getAttributesByGroup(groupId).subscribe({
      next: (attributes) => {
        this.attributes = attributes;
      },
      error: (error) => {
        this.error = 'Failed to load attributes';
      }
    });
  }

  onSubmit(): void {
    if (this.groupForm.valid) {
      this.submitting = true;
      this.error = null;

      const groupData = this.groupForm.value;
      const id = this.route.snapshot.paramMap.get('id');

      const request = id
        ? this.attributeGroupService.updateGroup(parseInt(id), groupData)
        : this.attributeGroupService.createGroup(groupData);

      request.subscribe({
        next: (group) => {
          if (!id) {
            if (group && group.id) {
              this.router.navigate(['/settings/attribute-groups', group.id, 'edit']);
            } else {
              this.router.navigate(['/settings/attribute-groups']);
            }
          }
          this.submitting = false;
        },
        error: (error) => {
          this.error = 'Failed to save group';
          this.submitting = false;
        }
      });
    }
  }

  onAddAttribute(): void {
    if (this.attributeForm.valid) {
      const groupId = this.route.snapshot.paramMap.get('id');
      if (!groupId) {
        this.error = 'Please save the group first before adding attributes';
        return;
      }

      const attributeData = {
        ...this.attributeForm.value,
        group: parseInt(groupId)
      };

      this.radiusAttributeService.createAttribute(attributeData).subscribe({
        next: (attribute) => {
          this.attributes.push(attribute);
          this.attributeForm.reset({
            vendor_id: 0,
            attribute_type: 'string'
          });
        },
        error: (error) => {
          this.error = 'Failed to add attribute';
        }
      });
    }
  }

  onDeleteAttribute(id: number): void {
    if (confirm('Are you sure you want to delete this attribute?')) {
      this.radiusAttributeService.deleteAttribute(id).subscribe({
        next: () => {
          this.attributes = this.attributes.filter(attr => attr.id !== id);
        },
        error: (error) => {
          this.error = 'Failed to delete attribute';
        }
      });
    }
  }

  onCancel(): void {
    this.router.navigate(['/settings/attribute-groups']);
  }

  isFieldInvalid(fieldName: string): boolean {
    const field = this.groupForm.get(fieldName);
    return field ? field.invalid && (field.dirty || field.touched) : false;
  }

  getErrorMessage(fieldName: string): string {
    const field = this.groupForm.get(fieldName);
    if (!field) return '';

    if (field.errors?.['required']) {
      return `${fieldName} is required`;
    }
    if (field.errors?.['maxlength']) {
      return `${fieldName} must be less than 255 characters`;
    }
    return '';
  }
} 