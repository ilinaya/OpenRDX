import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { UserGroupService } from '../../../../shared/services/user-group.service';
import { UserGroup } from '../../../../shared/models/user-group.model';

@Component({
  selector: 'app-user-group-form',
  templateUrl: './user-group-form.component.html',
  styleUrls: ['./user-group-form.component.scss']
})
export class UserGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  groups: UserGroup[] = [];
  isEditMode = false;
  loading = false;
  submitting = false;
  error: string | null = null;

  constructor(
    private fb: FormBuilder,
    private userGroupService: UserGroupService,
    private route: ActivatedRoute,
    private router: Router
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      parent: [null]
    });
  }

  ngOnInit(): void {
    this.loadGroups();
    const id = this.route.snapshot.paramMap.get('id');
    if (id) {
      this.isEditMode = true;
      this.loadGroup(parseInt(id));
    }
  }

  loadGroups(): void {
    this.userGroupService.getGroupTree().subscribe({
      next: (groups) => {
        this.groups = groups;
      },
      error: (error) => {
        this.error = 'Failed to load groups';
      }
    });
  }

  loadGroup(id: number): void {
    this.loading = true;
    this.userGroupService.getGroup(id).subscribe({
      next: (group) => {
        this.groupForm.patchValue({
          name: group.name,
          description: group.description,
          parent: group.parent?.id || null
        });
        this.loading = false;
      },
      error: (error) => {
        this.error = 'Failed to load group';
        this.loading = false;
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
        ? this.userGroupService.updateGroup(parseInt(id), groupData)
        : this.userGroupService.createGroup(groupData);

      request.subscribe({
        next: () => {
          this.router.navigate(['/settings/user-groups']);
        },
        error: (error) => {
          this.error = 'Failed to save group';
          this.submitting = false;
        }
      });
    }
  }

  onCancel(): void {
    this.router.navigate(['/settings/user-groups']);
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